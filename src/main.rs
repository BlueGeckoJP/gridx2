mod config;
mod directory;
mod image;
mod session;
mod ui;
mod utils;

use crate::config::app_config::AppConfig;
use crate::image::cache::ImageCache;
use crate::session::Session;
use crate::ui::builder::build_ui;
use gtk4 as gtk;
use gtk4::prelude::{ApplicationExt, ApplicationExtManual};
use gtk4::{Application, glib};

fn main() -> glib::ExitCode {
    gtk::init().expect("Failed to initialize GTK");

    let dark_mode = ui::theme::is_gtk_dark_theme();
    let app_config = AppConfig::init(dark_mode);

    let image_cache = ImageCache::new(5000);

    let session = Session::new();

    let app = Application::builder()
        .application_id("me.bluegecko.gridx2")
        .build();

    app.connect_activate(move |app| {
        build_ui(
            app,
            app_config.clone(),
            image_cache.clone(),
            session.clone(),
        );
    });

    app.run()
}
