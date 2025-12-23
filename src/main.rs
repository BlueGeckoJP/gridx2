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
use std::sync::{Arc, Mutex, RwLock, RwLockReadGuard};

type ImageCache = LruCache<String, Arc<Texture>>;

struct AppState {
    original_dir: RwLock<String>,
    dir_entries: RwLock<Vec<entry::DirEntry>>,
    shared: Arc<Shared>,
}

struct Shared {
    config: RwLock<AppConfig>,
    image_cache: Mutex<ImageCache>,
}

impl Shared {
    fn new() -> Self {
        Self {
            config: RwLock::new(AppConfig::load().unwrap_or_default()),
            image_cache: Mutex::new(LruCache::new(
                NonZero::new(5000).expect("Failed to create NonZero value"),
            )),
        }
    }

    pub fn config(&self) -> Result<RwLockReadGuard<'_, AppConfig>, String> {
        self.config
            .read()
            .map_err(|e| format!("Failed to lock config: {}", e))
    }

    pub fn update_config<F>(&self, update_fn: F) -> Result<(), String>
    where
        F: FnOnce(&mut AppConfig),
    {
        let mut config = self
            .config
            .write()
            .map_err(|e| format!("Failed to lock config for write: {}", e))?;
        update_fn(&mut config);
        Ok(())
    }

    pub fn image_cache(&self) -> Result<std::sync::MutexGuard<'_, ImageCache>, String> {
        self.image_cache
            .lock()
            .map_err(|e| format!("Failed to lock image cache: {}", e))
    }
}

pub struct AppUI {
    top_vbox: gtk::Box,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            original_dir: RwLock::new(String::new()),
            dir_entries: RwLock::new(Vec::new()),
            shared: Arc::new(Shared::new()),
        }
    }
}

impl AppState {
    pub fn original_dir(&self) -> Result<String, String> {
        self.original_dir
            .read()
            .map(|dir| dir.clone())
            .map_err(|e| format!("Failed to lock original_dir: {}", e))
    }

    pub fn set_original_dir(&self, dir: String) -> Result<(), String> {
        self.original_dir
            .write()
            .map(|mut d| *d = dir)
            .map_err(|e| format!("Failed to lock original_dir for write: {}", e))
    }

    pub fn dir_entries(&self) -> Result<Vec<entry::DirEntry>, String> {
        self.dir_entries
            .read()
            .map(|entries| entries.clone())
            .map_err(|e| format!("Failed to lock dir_entries: {}", e))
    }

    pub fn set_dir_entries(&self, entries: Vec<entry::DirEntry>) -> Result<(), String> {
        self.dir_entries
            .write()
            .map(|mut e| *e = entries)
            .map_err(|e| format!("Failed to lock dir_entries for write: {}", e))
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
