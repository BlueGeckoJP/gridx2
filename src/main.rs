mod accordion_widget;
mod app_config;
mod entry;
mod image_cache;
mod image_widget;
mod settings_window;

use crate::accordion_widget::AccordionWidget;
use crate::app_config::AppConfig;
use crate::image_cache::ImageCache;
use crate::image_widget::ImageWidget;
use crate::settings_window::SettingsWindow;
use anyhow::{anyhow, Result};
use gtk4 as gtk;
use gtk4::gio::Cancellable;
use gtk4::prelude::{
    ActionMapExt, ApplicationExt, ApplicationExtManual, ApplicationWindowExt, BoxExt, FileExt,
    GtkApplicationExt, GtkWindowExt, WidgetExt,
};
use gtk4::{gdk, gio, glib, Application, ApplicationWindow, CssProvider, FileDialog};
use regex::Regex;
use std::cell::RefCell;
use std::cmp::{min, Ordering};
use std::path::Path;
use std::process::Command;
use std::rc::Rc;
use std::sync::{Arc, LazyLock, Mutex};
use std::time::Duration;

static APP_CONFIG: LazyLock<Mutex<AppConfig>> =
    LazyLock::new(|| Mutex::new(AppConfig::load().unwrap_or_default()));
static IMAGE_CACHE: LazyLock<Mutex<ImageCache>> =
    LazyLock::new(|| Mutex::new(ImageCache::new(500)));

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

fn build_ui(app: &Application) {
    load_css();

    let app_state = Arc::new(Mutex::new(AppState::new()));

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

    // Build menubar
    let menubar = gio::Menu::new();

    let file_menu = gio::Menu::new();
    file_menu.append(Some("Open Folder"), Some("app.open"));
    file_menu.append(Some("Open Settings"), Some("app.settings"));

    menubar.append_submenu(Some("File"), &file_menu);

    app.set_menubar(Some(&menubar));
    window.set_show_menubar(true);

    let open_action = gio::SimpleAction::new("open", None);
    open_action.connect_activate(glib::clone!(
        #[weak]
        window,
        move |_, _| {
            let dialog = FileDialog::new();
            let cancellable = Cancellable::new();
            let app_ui = app_ui.clone();
            let app_state = app_state.clone();
            dialog.select_folder(Some(&window), Some(&cancellable), move |result| {
                if let Ok(path) = result {
                    if let Some(dir) = path.path() {
                        let mut app_state_guard = app_state.lock().unwrap();
                        app_state_guard.original_dir = dir.to_str().unwrap().to_string();
                        let app_state = app_state.clone();
                        glib::spawn_future_local(async move {
                            update_entry(app_state.clone(), &app_ui.borrow().top_vbox)
                                .expect("Failed to update entry");
                        });
                    }
                }
            });
        }
    ));
    app.add_action(&open_action);

    let settings_action = gio::SimpleAction::new("settings", None);
    settings_action.connect_activate(glib::clone!(
        #[weak]
        window,
        move |_, _| {
            let settings_window = SettingsWindow::new(&window);
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

fn update_entry(app_state: Arc<Mutex<AppState>>, vbox: &gtk::Box) -> Result<()> {
    while let Some(child) = vbox.first_child() {
        vbox.remove(&child);
    }

    let dir_path = {
        let app_state_guard = app_state
            .lock()
            .map_err(|_| anyhow::anyhow!("Failed to lock"))?;
        app_state_guard.original_dir.clone()
    };

    let entries = entry::DirEntry::search(dir_path);

    match entries {
        Ok(dir_entries) => {
            let (original_dir, mut entries_indies) = {
                let mut app_state_guard = app_state
                    .lock()
                    .map_err(|_| anyhow::anyhow!("Failed to lock"))?;

                app_state_guard.dir_entries = dir_entries;

                let original_dir = app_state_guard.original_dir.clone();
                let dir_entries = app_state_guard.dir_entries.clone();

                (original_dir, dir_entries)
            };

            entries_indies.sort_by(|a, b| a.dir_path.cmp(&b.dir_path));

            for (index, entry) in entries_indies.iter().enumerate() {
                let rel_path = get_relative_path(&original_dir, &entry.dir_path)?;
                let accordion_widget =
                    Rc::new(RefCell::new(AccordionWidget::new(rel_path.as_str())));
                let mut overlays = Vec::new();

                for _ in 0..entry.image_entries.len() {
                    let thumbnail_size = {
                        let app_config = APP_CONFIG
                            .lock()
                            .map_err(|_| anyhow!("Failed to lock app config"))?;
                        app_config.thumbnail_size
                    } as i32;

                    let fixed_size_container = gtk::Box::new(gtk::Orientation::Vertical, 0);
                    fixed_size_container.set_size_request(thumbnail_size, thumbnail_size);
                    fixed_size_container.set_halign(gtk::Align::Center);
                    fixed_size_container.set_valign(gtk::Align::Center);

                    let overlay = gtk::Overlay::new();
                    overlay.set_child(Some(&fixed_size_container));

                    accordion_widget.borrow_mut().flow_box.append(&overlay);
                    overlays.push(overlay);
                }

                vbox.append(&accordion_widget.borrow().widget);

                let app_state_clone = app_state.clone();

                accordion_widget
                    .clone()
                    .borrow()
                    .connect_expanded(move |is_expanded| {
                        if is_expanded {
                            let app_state_clone = app_state_clone.clone();
                            let accordion_widget = accordion_widget.clone();
                            let overlays = overlays.clone();

                            while let Some(child) = accordion_widget.borrow().flow_box.first_child()
                            {
                                accordion_widget.borrow().flow_box.remove(&child);
                            }

                            glib::spawn_future_local(async move {
                                let dir_entry_clone = {
                                    match app_state_clone.lock() {
                                        Ok(app) => {
                                            if index >= app.dir_entries.len() {
                                                eprintln!("Invalid index: {index}");
                                                return;
                                            }

                                            app.dir_entries.clone()[index].clone()
                                        }
                                        Err(e) => {
                                            eprintln!("Failed to lock app state: {e}");
                                            return;
                                        }
                                    }
                                };

                                let mut loaded_entry = dir_entry_clone;
                                if let Err(e) = loaded_entry.load_images() {
                                    eprintln!("Failed to load images: {e}");
                                    return;
                                }

                                let mut image_entries = loaded_entry.image_entries.clone();
                                image_entries.sort_by(|a, b| {
                                    natural_sort(a.image_path.as_str(), b.image_path.as_str())
                                        .unwrap_or(Ordering::Equal)
                                });

                                for (index, image_entry) in image_entries.iter().enumerate() {
                                    if let Some(img) = &image_entry.image {
                                        let mut image_widget = ImageWidget::new();
                                        image_widget
                                            .set_image(&image_entry.image_path, img.clone());

                                        let accordion_widget = accordion_widget.clone();
                                        let overlays = overlays.clone();

                                        glib::MainContext::default().spawn_local(async move {
                                            if index < overlays.len() {
                                                let overlay = overlays[index].clone();
                                                overlay.add_overlay(image_widget.widget());
                                                accordion_widget
                                                    .borrow_mut()
                                                    .flow_box
                                                    .append(&overlay);
                                            }
                                        });

                                        if index % 5 == 0 {
                                            glib::timeout_future(Duration::from_millis(10)).await;
                                        }
                                    }
                                }
                            });
                        }
                    });
            }

            Ok(())
        }
        Err(e) => {
            println!("Error: {e}");
            Err(e)
        }
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

fn get_relative_path(base_path: &str, path: &str) -> Result<String> {
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

fn open_with_xdg_open(image_path: String) -> Result<()> {
    let mut open_command = {
        let app_config = APP_CONFIG
            .lock()
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

    cmd.spawn()?;

    Ok(())
}

fn natural_sort(a: &str, b: &str) -> Result<Ordering> {
    let re_all = Regex::new(r"(\d+)|(\D+)")?;
    let re_num = Regex::new(r"^\d+$")?;

    let a_parts: Vec<&str> = re_all.find_iter(a).map(|m| m.as_str()).collect();
    let b_parts: Vec<&str> = re_all.find_iter(b).map(|m| m.as_str()).collect();

    for i in 0..min(a_parts.len(), b_parts.len()) {
        let a_part = a_parts[i];
        let b_part = b_parts[i];

        if i >= a_parts.len() {
            return Ok(Ordering::Less);
        }
        if i >= b_parts.len() {
            return Ok(Ordering::Greater);
        }

        if a_part == b_part {
            continue;
        }

        return if re_num.is_match(a_part) {
            if re_num.is_match(b_part) {
                let a_num = a_part.parse::<i32>()?;
                let b_num = b_part.parse::<i32>()?;
                Ok(a_num.cmp(&b_num))
            } else {
                Ok(Ordering::Greater)
            }
        } else if re_num.is_match(b_part) {
            Ok(Ordering::Less)
        } else {
            Ok(a_part.cmp(b_part))
        };
    }

    Ok(Ordering::Equal)
}
