use gtk4 as gtk;
use gtk4::Picture;
use gtk4::gdk::Texture;
use gtk4::prelude::{BoxExt, TextureExt, WidgetExt};

#[derive(Clone)]
pub struct ImageWidget {
    widget: gtk::Box,
    picture: Picture,
}

impl ImageWidget {
    pub fn new() -> Self {
        let widget = gtk::Box::new(gtk::Orientation::Vertical, 0);
        widget.set_halign(gtk::Align::Center);
        widget.set_valign(gtk::Align::Center);

        let picture = Picture::new();
        picture.set_halign(gtk::Align::Center);
        picture.set_valign(gtk::Align::Center);

        widget.append(&picture);

        Self { widget, picture }
    }

    pub fn connect_clicked<F: Fn() + 'static>(&self, callback: F) {
        let click_gesture = gtk::GestureClick::new();
        click_gesture.connect_released(move |_gesture, _n_press, _x, _y| {
            callback();
        });

        self.picture.add_controller(click_gesture);
    }

    pub fn set_image(&self, texture: &Texture) {
        self.picture.set_paintable(Some(texture));
        self.picture
            .set_size_request(texture.width(), texture.height());
    }

    pub fn widget(&self) -> &gtk::Box {
        &self.widget
    }
}
