//! The responsibility: build the main application window and expose its high-level actions.

use gtk4::{
    Application, ApplicationWindow, Button, HeaderBar, Label,
    gio::{self, prelude::ActionMapExt},
    glib,
    prelude::{ActionableExt, ButtonExt, GtkWindowExt},
};

/// Main GTK application window and its primary content container.
///
/// This type owns the shell widgets and exposes action hooks without knowing what those actions do.
pub struct MainWindow {
    app: Application,
    window: gtk4::ApplicationWindow,
    container: gtk4::Box,
}

impl MainWindow {
    /// Creates and presents the main window with open/settings controls and a scrollable content area.
    pub fn new(app: &Application) -> Self {
        let window = ApplicationWindow::builder()
            .application(app)
            .default_width(800)
            .default_height(600)
            .build();

        let container = gtk4::Box::new(gtk4::Orientation::Vertical, 5);

        let header_bar = HeaderBar::new();
        header_bar.set_title_widget(Some(&Label::new(Some("gridx2"))));

        let open_button = Button::new();
        let open_icon = gtk4::Image::from_icon_name("document-open-symbolic");
        open_button.set_child(Some(&open_icon));
        open_button.set_action_name(Some("app.open"));

        let settings_button = Button::new();
        let settings_icon = gtk4::Image::from_icon_name("preferences-system-symbolic");
        settings_button.set_child(Some(&settings_icon));
        settings_button.set_action_name(Some("app.settings"));

        header_bar.pack_start(&open_button);
        header_bar.pack_end(&settings_button);

        window.set_titlebar(Some(&header_bar));

        let scrollable_window = gtk4::ScrolledWindow::builder()
            .hscrollbar_policy(gtk4::PolicyType::Never)
            .vscrollbar_policy(gtk4::PolicyType::Automatic)
            .child(&container)
            .build();

        window.set_child(Some(&scrollable_window));

        window.present();

        Self {
            app: app.clone(),
            window,
            container,
        }
    }

    /// Registers the callback invoked by the window's open action.
    ///
    /// The callback receives the GTK window for dialog parenting and the content container to refresh.
    pub fn set_open_callback<F: Fn(&ApplicationWindow, &gtk4::Box) + 'static>(&self, callback: F) {
        let window = self.window.clone();
        let container = self.container.clone();

        let open_action = gio::SimpleAction::new("open", None);
        open_action.connect_activate(glib::clone!(
            #[weak]
            window,
            #[weak]
            container,
            move |_, _| {
                callback(&window, &container);
            }
        ));

        self.app.add_action(&open_action);
    }

    /// Registers the callback invoked by the window's settings action.
    pub fn set_settings_callback<F: Fn(&ApplicationWindow) + 'static>(&self, callback: F) {
        let window = self.window.clone();

        let settings_action = gio::SimpleAction::new("settings", None);
        settings_action.connect_activate(glib::clone!(
            #[weak]
            window,
            move |_, _| {
                callback(&window);
            }
        ));

        self.app.add_action(&settings_action);
    }
}
