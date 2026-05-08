mod config;
mod entry;
mod file_utils;
mod image_cache;
mod image_entry;
mod image_loader;
mod session;
mod settings_window;
mod theme;
mod ui_builder;
mod utils;
mod widgets;

use crate::config::app_config::AppConfig;
use crate::file_utils::{get_relative_path, search_and_prepare_entries};
use crate::image_cache::ImageCache;
use crate::session::Session;
use crate::ui_builder::{build_ui, create_blank_accordion_widget};
use gtk4 as gtk;
use gtk4::prelude::{ApplicationExt, ApplicationExtManual, BoxExt, WidgetExt};
use gtk4::{Application, CssProvider, gdk, glib};

pub struct AppUI {
    top_vbox: gtk::Box,
}

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

fn update_entry(
    session: Session,
    app_config: AppConfig,
    image_cache: ImageCache,
    vbox: &gtk::Box,
) -> eyre::Result<()> {
    clear_ui(vbox);

    let config = app_config.get()?;
    let max_depth = config.max_depth;

    search_and_prepare_entries(session.clone(), max_depth)?;

    let (original_dir, dir_entries) = (session.original_dir()?, session.dir_entries()?);

    for (index, entry) in dir_entries.iter().enumerate() {
        let title = format!(
            "{} | {} entries",
            get_relative_path(&original_dir, &entry.dir_path)?,
            entry.image_entries.len()
        );

        create_blank_accordion_widget(
            vbox,
            entry.image_entries.len(),
            &title,
            index,
            session.clone(),
            app_config.clone(),
            image_cache.clone(),
        )?;
    }

    Ok(())
}

fn clear_ui(vbox: &gtk::Box) {
    while let Some(child) = vbox.first_child() {
        vbox.remove(&child);
    }
}

fn load_css() {
    let provider = CssProvider::new();
    provider.load_from_data(include_str!("style.css"));

    gtk::style_context_add_provider_for_display(
        &gdk::Display::default().expect("Failed to get display"),
        &provider,
        gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );
}
