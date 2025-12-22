mod accordion_widget;
mod app_config;
mod entry;
mod image_entry;
mod image_loader;
mod image_widget;
mod settings_window;
mod ui_builder;
mod utils;

use crate::app_config::AppConfig;
use crate::ui_builder::{build_ui, create_blank_accordion_widget};
use anyhow::anyhow;
use gtk4 as gtk;
use gtk4::gdk::Texture;
use gtk4::prelude::{ApplicationExt, ApplicationExtManual, BoxExt, WidgetExt};
use gtk4::{Application, CssProvider, gdk, glib};
use lru::LruCache;
use std::cmp::Ordering;
use std::num::NonZero;
use std::path::Path;
use std::process::{Command, Stdio};
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

fn search_and_prepare_entries(
    app_state: Arc<Mutex<AppState>>,
) -> anyhow::Result<(String, Vec<entry::DirEntry>)> {
    let dir_path = {
        let app_state_guard = app_state.lock().map_err(|_| anyhow!("Failed to lock"))?;
        app_state_guard.original_dir.clone()
    };

    let entries = entry::DirEntry::search(&dir_path)?;

    let mut app_state_guard = app_state.lock().map_err(|_| anyhow!("Failed to lock"))?;

    app_state_guard.dir_entries = entries;
    app_state_guard
        .dir_entries
        .sort_by(|a, b| a.dir_path.cmp(&b.dir_path));

    let original_dir = app_state_guard.original_dir.clone();
    let dir_entries = app_state_guard.dir_entries.clone();

    Ok((original_dir, dir_entries))
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

fn get_relative_path(base_path: &str, path: &str) -> anyhow::Result<String> {
    let base_path = Path::new(base_path).canonicalize()?;
    let path = Path::new(path).canonicalize()?;
    let relative_path = path.strip_prefix(&base_path)?;
    let relative_path = relative_path.to_str().ok_or_else(|| {
        anyhow::anyhow!(
            "Failed to convert path to string: {:?}",
            relative_path.to_str()
        )
    })?;

    if relative_path.is_empty() {
        return Ok(String::from("."));
    }

    Ok(relative_path.to_string())
}

fn open_with_xdg_open(image_path: String) -> anyhow::Result<()> {
    let mut open_command = {
        let app_config = APP_CONFIG
            .read()
            .map_err(|_| anyhow!("Failed to lock app config"))?;
        app_config.open_command.clone()
    };
    let index = open_command.iter().position(|x| x == &"<path>".to_string());

    let mut cmd = match index {
        Some(index) => {
            open_command[index] = image_path.clone();
            let first_arg = open_command[0].clone();
            let mut cmd = Command::new(&first_arg);
            cmd.args(&open_command[1..]);
            cmd
        }
        None => {
            let app_config = AppConfig::default();
            let first_arg = app_config.open_command[0].clone();
            let mut cmd = Command::new(&first_arg);
            cmd.args(&app_config.open_command[1..]);
            cmd
        }
    };

    cmd.stdout(Stdio::null())
        .stderr(Stdio::null())
        .stdin(Stdio::null());

    cmd.spawn()?;

    Ok(())
}

fn sort_by_updated_at(a: &str, b: &str, descending: bool) -> anyhow::Result<Ordering> {
    let a_metadata = std::fs::metadata(a)?;
    let b_metadata = std::fs::metadata(b)?;

    let a_modified = a_metadata.modified()?;
    let b_modified = b_metadata.modified()?;

    Ok(match descending {
        true => a_modified.cmp(&b_modified),
        false => b_modified.cmp(&a_modified),
    })
}
