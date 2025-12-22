mod accordion_widget;
mod app_config;
mod entry;
mod file_utils;
mod image_entry;
mod image_loader;
mod image_widget;
mod settings_window;
mod ui_builder;
mod utils;

use crate::app_config::AppConfig;
use crate::file_utils::{get_relative_path, search_and_prepare_entries};
use crate::ui_builder::{build_ui, create_blank_accordion_widget};
use gtk4 as gtk;
use gtk4::gdk::Texture;
use gtk4::prelude::{ApplicationExt, ApplicationExtManual, BoxExt, WidgetExt};
use gtk4::{Application, CssProvider, gdk, glib};
use lru::LruCache;
use std::num::NonZero;
use std::sync::{Arc, LazyLock, Mutex, RwLock};

static APP_CONFIG: LazyLock<RwLock<AppConfig>> =
    LazyLock::new(|| RwLock::new(AppConfig::load().unwrap_or_default()));
static IMAGE_CACHE: LazyLock<Mutex<LruCache<String, Arc<Texture>>>> =
    LazyLock::new(|| Mutex::new(LruCache::new(NonZero::new(5000).unwrap())));

struct AppState {
    original_dir: String,
    dir_entries: Vec<entry::DirEntry>,
}

struct AppUI {
    top_vbox: gtk::Box,
}

impl AppState {
    fn new() -> Self {
        Self {
            original_dir: String::from("."),
            dir_entries: Vec::new(),
        }
    }
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

fn update_entry(app_state: Arc<Mutex<AppState>>, vbox: &gtk::Box) -> anyhow::Result<()> {
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
