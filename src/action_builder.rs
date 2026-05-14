use gtk4::{
    FileDialog,
    gio::{Cancellable, prelude::FileExt},
    glib,
};

use crate::{
    config::app_config::AppConfig,
    image_cache::ImageCache,
    session::Session,
    ui::{main_window::MainWindow, settings_window::SettingsWindow},
    ui_builder::update_entry,
};

pub fn setup_main_window_callbacks(
    main_window: &MainWindow,
    session: Session,
    app_config: AppConfig,
    image_cache: ImageCache,
) {
    let app_config_for_open = app_config.clone();
    main_window.set_open_callback(move |window, container| {
        let dialog = FileDialog::new();
        let cancellable = Cancellable::new();

        let session = session.clone();
        let app_config = app_config_for_open.clone();
        let image_cache = image_cache.clone();
        let container = container.clone();

        dialog.select_folder(Some(window), Some(&cancellable), move |result| {
            let path = match result {
                Ok(path) => match path.path() {
                    Some(dir) => dir.to_string_lossy().to_string(),
                    None => {
                        eprintln!("No directory selected");
                        return;
                    }
                },
                Err(e) => {
                    eprintln!("Failed to open file dialog: {e}");
                    return;
                }
            };

            if let Err(e) = session.set_original_dir(path) {
                eprintln!("Failed to set original_dir: {e}");
                return;
            }

            glib::spawn_future_local(async move {
                if let Err(e) = update_entry(
                    session.clone(),
                    app_config.clone(),
                    image_cache.clone(),
                    container.clone(),
                ) {
                    eprintln!("Failed to update entry: {e}");
                }
            });
        });
    });

    main_window.set_settings_callback(move |window| {
        let config = match app_config.get() {
            Ok(config) => config,
            Err(e) => {
                eprintln!("Failed to get app config: {e}");
                return;
            }
        };

        let app_config = app_config.clone();
        let settings_window = SettingsWindow::new(window, config, move |config| {
            if let Err(e) = app_config.update(config) {
                eprintln!("Failed to save config: {e}");
            }
        });

        match settings_window {
            Ok(settings_window) => {
                settings_window.show();
            }
            Err(e) => {
                eprintln!("Failed to create settings window: {e}");
            }
        }
    });
}
