use gtk4::gio::Cancellable;
use gtk4::gio::prelude::{ActionMapExt, FileExt};
use gtk4::prelude::{ActionableExt, BoxExt, ButtonExt, GtkWindowExt, WidgetExt};
use gtk4::{self as gtk, Button, FileDialog, HeaderBar, gio, glib};
use gtk4::{Application, ApplicationWindow};
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

use crate::config::app_config::AppConfig;
use crate::image_cache::ImageCache;
use crate::image_loader::load_and_display_images;
use crate::settings_window::SettingsWindow;
use crate::widgets::accordion_widget::AccordionWidget;
use crate::{AppState, AppUI, load_css, update_entry};

pub fn build_ui(app: &Application, app_config: AppConfig, image_cache: ImageCache) {
    load_css();

    let app_state = Arc::new(AppState::default());

    let window = ApplicationWindow::builder()
        .application(app)
        .default_width(800)
        .default_height(600)
        .title("gridx2")
        .build();

    // Build layout
    let vbox = gtk::Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .spacing(5)
        .build();

    let app_ui = Rc::new(RefCell::new(AppUI {
        top_vbox: vbox.clone(),
    }));

    // Build header bar
    let header_bar = HeaderBar::new();
    header_bar.set_title_widget(Some(&gtk::Label::new(Some("gridx2"))));

    let button_open = Button::new();
    let icon_open = gtk::Image::from_icon_name("document-open-symbolic");
    button_open.set_child(Some(&icon_open));
    button_open.set_action_name(Some("app.open"));

    let button_settings = Button::new();
    let icon_settings = gtk::Image::from_icon_name("preferences-system-symbolic");
    button_settings.set_child(Some(&icon_settings));
    button_settings.set_action_name(Some("app.settings"));

    header_bar.pack_start(&button_open);
    header_bar.pack_end(&button_settings);

    window.set_titlebar(Some(&header_bar));

    // Build actions
    build_action(
        app,
        &window,
        &app_ui,
        &app_state,
        app_config.clone(),
        image_cache.clone(),
    );

    // Build a scrollable window
    let scrollable_window = gtk::ScrolledWindow::builder()
        .hscrollbar_policy(gtk::PolicyType::Never)
        .vscrollbar_policy(gtk::PolicyType::Automatic)
        .child(&vbox)
        .build();

    window.set_child(Some(&scrollable_window));

    // Finalize
    window.present();
}

fn build_action(
    app: &Application,
    window: &ApplicationWindow,
    app_ui: &Rc<RefCell<AppUI>>,
    app_state: &Arc<AppState>,
    app_config: AppConfig,
    image_cache: ImageCache,
) {
    let app_ui = app_ui.clone();
    let app_state_clone = app_state.clone();
    let app_config_clone = app_config.clone();

    let open_action = gio::SimpleAction::new("open", None);
    open_action.connect_activate(glib::clone!(
        #[weak]
        window,
        #[strong]
        app_ui,
        #[strong]
        app_state_clone,
        #[strong]
        image_cache,
        move |_, _| {
            let dialog = FileDialog::new();
            let cancellable = Cancellable::new();
            let app_ui = app_ui.clone();
            let app_state_clone = app_state_clone.clone();
            let app_config = app_config_clone.clone();
            let image_cache = image_cache.clone();
            dialog.select_folder(Some(&window), Some(&cancellable), move |result| {
                if let Ok(path) = result
                    && let Some(dir) = path.path()
                    && let Some(dir_str) = dir.to_str()
                {
                    if let Err(e) = app_state_clone
                        .update_runtime_ctx(|ctx| ctx.original_dir = Arc::new(dir_str.to_string()))
                    {
                        eprintln!("Failed to set original_dir: {}", e);
                        return;
                    }
                    let app_state = app_state_clone.clone();
                    let app_config = app_config.clone();
                    let image_cache = image_cache.clone();
                    glib::spawn_future_local(async move {
                        if let Err(e) = update_entry(
                            app_state.clone(),
                            app_config.clone(),
                            image_cache.clone(),
                            &app_ui.borrow().top_vbox,
                        ) {
                            eprintln!("Failed to update entry: {}", e);
                        }
                    });
                }
            });
        }
    ));
    app.add_action(&open_action);

    let settings_action = gio::SimpleAction::new("settings", None);
    settings_action.connect_activate(glib::clone!(
        #[weak]
        window,
        #[strong]
        app_config,
        move |_, _| {
            let settings_window = SettingsWindow::new(&window, app_config.clone());
            match settings_window {
                Ok(settings_window) => {
                    settings_window.show();
                }
                Err(e) => {
                    eprintln!("Failed to create settings window: {e}");
                }
            }
        }
    ));
    app.add_action(&settings_action);
}

pub fn create_blank_accordion_widget(
    vbox: &gtk::Box,
    count: usize,
    title: &str,
    index: usize,
    app_state: Arc<AppState>,
    app_config: AppConfig,
    image_cache: ImageCache,
) -> eyre::Result<()> {
    let dark_mode = match app_config.get() {
        Ok(config) => config.dark_mode,
        Err(e) => {
            eprintln!("Failed to get app config: {e}");
            false
        }
    };

    let accordion_widget = Rc::new(RefCell::new(AccordionWidget::new(title, dark_mode)?));
    let mut overlays = Vec::new();

    let config = app_config.get()?;
    let thumbnail_size = config.thumbnail_size as i32;

    for _ in 0..count {
        let fixed_size_container = gtk::Box::new(gtk::Orientation::Vertical, 0);
        fixed_size_container.set_size_request(thumbnail_size, thumbnail_size);
        fixed_size_container.set_halign(gtk::Align::Center);
        fixed_size_container.set_valign(gtk::Align::Center);

        let overlay = gtk::Overlay::new();
        overlay.set_child(Some(&fixed_size_container));

        accordion_widget.borrow().flow_box.append(&overlay);
        overlays.push(overlay);
    }

    vbox.append(&accordion_widget.borrow().widget);

    setup_accordion_expand_handler(
        index,
        accordion_widget,
        overlays,
        app_state,
        app_config,
        image_cache,
    );

    Ok(())
}

fn setup_accordion_expand_handler(
    index: usize,
    accordion_widget: Rc<RefCell<AccordionWidget>>,
    overlays: Vec<gtk::Overlay>,
    app_state: Arc<AppState>,
    app_config: AppConfig,
    image_cache: ImageCache,
) {
    accordion_widget
        .clone()
        .borrow()
        .connect_expanded(move |is_expanded| {
            if is_expanded {
                let app_state_clone = app_state.clone();
                let app_config = app_config.clone();
                let image_cache = image_cache.clone();
                let accordion_widget = accordion_widget.clone();
                let overlays = overlays.clone();

                prepare_accordion_for_loading(&accordion_widget);

                glib::spawn_future_local(async move {
                    load_and_display_images(
                        app_state_clone,
                        image_cache,
                        app_config.clone(),
                        accordion_widget,
                        overlays,
                        index,
                    )
                    .await;
                });
            }
        });
}

fn prepare_accordion_for_loading(accordion_widget: &Rc<RefCell<AccordionWidget>>) {
    let accordion_widget = accordion_widget.borrow();

    while let Some(child) = accordion_widget.flow_box.first_child() {
        accordion_widget.flow_box.remove(&child);
    }

    accordion_widget.progress_bar.set_fraction(0.0);
    accordion_widget.progress_bar.set_visible(true);
}
