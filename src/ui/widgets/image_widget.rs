//! The responsibility: provide a clickable thumbnail image widget.

use gtk4 as gtk;
use gtk4::Picture;
use gtk4::gdk::Texture;
use gtk4::gio::{self, prelude::ActionMapExt};
use gtk4::prelude::{BoxExt, GestureSingleExt, PopoverExt, TextureExt, WidgetExt};

const PRIMARY_BUTTON: u32 = 1;
const SECONDARY_BUTTON: u32 = 3;
const ACTION_GROUP_NAME: &str = "image";
const COPY_IMAGE_ACTION_NAME: &str = "copy-image";
const COPY_IMAGE_PATH_ACTION_NAME: &str = "copy-image-path";

/// GTK widget wrapper for displaying one thumbnail and handling click gestures.
///
/// Use this inside an image grid presenter rather than exposing raw `Picture` setup everywhere.
#[derive(Clone)]
pub struct ImageWidget {
    widget: gtk::Box,
    picture: Picture,
    context_menu: gtk::PopoverMenu,
    copy_image_action: gio::SimpleAction,
    copy_image_path_action: gio::SimpleAction,
}

impl ImageWidget {
    /// Creates an empty centered thumbnail widget.
    pub fn new() -> Self {
        let widget = gtk::Box::new(gtk::Orientation::Vertical, 0);
        widget.set_halign(gtk::Align::Center);
        widget.set_valign(gtk::Align::Center);

        let picture = Picture::new();
        picture.set_halign(gtk::Align::Center);
        picture.set_valign(gtk::Align::Center);

        widget.append(&picture);

        let action_group = gio::SimpleActionGroup::new();
        let copy_image_action = gio::SimpleAction::new(COPY_IMAGE_ACTION_NAME, None);
        let copy_image_path_action = gio::SimpleAction::new(COPY_IMAGE_PATH_ACTION_NAME, None);
        action_group.add_action(&copy_image_action);
        action_group.add_action(&copy_image_path_action);
        widget.insert_action_group(ACTION_GROUP_NAME, Some(&action_group));

        let context_menu = gtk::PopoverMenu::from_model(Some(&build_context_menu_model()));
        context_menu.set_has_arrow(false);
        context_menu.set_autohide(true);
        context_menu.set_position(gtk::PositionType::Bottom);
        context_menu.set_parent(&widget);

        let context_menu_for_gesture = context_menu.clone();
        let secondary_click_gesture = gtk::GestureClick::new();
        secondary_click_gesture.set_button(SECONDARY_BUTTON);
        secondary_click_gesture.connect_pressed(move |_gesture, _n_press, x, y| {
            let rect = gtk::gdk::Rectangle::new(x as i32, y as i32, 1, 1);
            context_menu_for_gesture.set_pointing_to(Some(&rect));
            context_menu_for_gesture.popup();
        });
        picture.add_controller(secondary_click_gesture);

        Self {
            widget,
            picture,
            context_menu,
            copy_image_action,
            copy_image_path_action,
        }
    }

    /// Registers a click callback for the displayed image.
    pub fn connect_clicked<F: Fn() + 'static>(&self, callback: F) {
        let click_gesture = gtk::GestureClick::new();
        click_gesture.set_button(PRIMARY_BUTTON);
        click_gesture.connect_released(move |_gesture, _n_press, _x, _y| {
            callback();
        });

        self.picture.add_controller(click_gesture);
    }

    /// Registers the callback invoked when the context menu requests that the image be copied.
    pub fn connect_copy_image_requested<F: Fn() + 'static>(&self, callback: F) {
        let context_menu = self.context_menu.clone();
        self.copy_image_action.connect_activate(move |_, _| {
            context_menu.popdown();
            callback();
        });
    }

    /// Registers the callback invoked when the context menu requests that the image path be copied.
    pub fn connect_copy_image_path_requested<F: Fn() + 'static>(&self, callback: F) {
        let context_menu = self.context_menu.clone();
        self.copy_image_path_action.connect_activate(move |_, _| {
            context_menu.popdown();
            callback();
        });
    }

    /// Sets the texture and sizes the picture to the decoded thumbnail dimensions.
    pub fn set_image(&self, texture: &Texture) {
        self.picture.set_paintable(Some(texture));
        self.picture
            .set_size_request(texture.width(), texture.height());
    }

    /// Returns the outer widget to append into GTK containers.
    pub fn widget(&self) -> &gtk::Box {
        &self.widget
    }
}

/// Builds the menu model used by the thumbnail context menu.
fn build_context_menu_model() -> gio::Menu {
    let menu = gio::Menu::new();
    menu.append(
        Some("Copy Image"),
        Some(&format!("{ACTION_GROUP_NAME}.{COPY_IMAGE_ACTION_NAME}")),
    );
    menu.append(
        Some("Copy Image Path"),
        Some(&format!(
            "{ACTION_GROUP_NAME}.{COPY_IMAGE_PATH_ACTION_NAME}"
        )),
    );
    menu
}
