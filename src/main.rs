mod action_builder;
mod config;
mod directory_section;
mod entry;
mod file_utils;
mod image_actions;
mod image_cache;
mod image_entry;
mod image_loader;
mod session;
mod theme;
mod ui;
mod ui_builder;
mod utils;

use crate::config::app_config::AppConfig;
use crate::image_cache::ImageCache;
use crate::session::Session;
use crate::ui_builder::build_ui;
use gtk4 as gtk;
use gtk4::prelude::{ApplicationExt, ApplicationExtManual};
use gtk4::{Application, glib};

fn main() -> glib::ExitCode {
    gtk::init().expect("Failed to initialize GTK");

    let dark_mode = theme::is_gtk_dark_theme();
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
