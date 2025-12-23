mod app_config;
mod app_state;
mod entry;
mod file_utils;
mod image_entry;
mod image_loader;
mod settings_window;
mod shared;
mod ui_builder;
mod utils;
mod widgets;

use std::sync::Arc;

use crate::app_state::AppState;
use crate::file_utils::{get_relative_path, search_and_prepare_entries};
use crate::ui_builder::{build_ui, create_blank_accordion_widget};
use gtk4 as gtk;
use gtk4::prelude::{ApplicationExt, ApplicationExtManual, BoxExt, WidgetExt};
use gtk4::{Application, CssProvider, gdk, glib};

pub struct AppUI {
    top_vbox: gtk::Box,
}

fn main() -> glib::ExitCode {
    gtk::init().expect("Failed to initialize GTK");

    let app = Application::builder()
        .application_id("me.bluegecko.gridx2")
        .build();

    app.connect_activate(move |app| {
        build_ui(app);
    });

    app.run()
}

fn update_entry(app_state: Arc<AppState>, vbox: &gtk::Box) -> anyhow::Result<()> {
    clear_ui(vbox);

    let (original_dir, entries_indies) = search_and_prepare_entries(app_state.clone())?;

    for (index, entry) in entries_indies.iter().enumerate() {
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
            app_state.clone(),
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
