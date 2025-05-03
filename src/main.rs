mod accordion_widget;
mod entry;
mod image_widget;
mod app_config;

use crate::accordion_widget::AccordionWidget;
use crate::image_widget::ImageWidget;
use anyhow::{anyhow, Result};
use gtk4 as gtk;
use gtk4::gio::Cancellable;
use gtk4::prelude::{
    ActionMapExt, ApplicationExt, ApplicationExtManual, ApplicationWindowExt, BoxExt, FileExt,
    GtkApplicationExt, GtkWindowExt, WidgetExt,
};
use gtk4::{gdk, gio, glib, Application, ApplicationWindow, CssProvider, FileDialog};
use std::cell::RefCell;
use std::path::Path;
use std::process::Command;
use std::rc::Rc;
use std::sync::{Arc, LazyLock, Mutex};
use crate::app_config::AppConfig;

static APP_CONFIG: LazyLock<Mutex<AppConfig>> = LazyLock::new(|| Mutex::new(AppConfig::default()));

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
            let (original_dir, entries_indies) = {
                let mut app_state_guard = app_state
                    .lock()
                    .map_err(|_| anyhow::anyhow!("Failed to lock"))?;

                app_state_guard.dir_entries = dir_entries;

                let original_dir = app_state_guard.original_dir.clone();
                let dir_entries = app_state_guard.dir_entries.clone();

                (original_dir, dir_entries)
            };

            for (index, entry) in entries_indies.iter().enumerate() {
                let rel_path = get_relative_path(&original_dir, &entry.dir_path)?;
                let accordion_widget =
                    Rc::new(RefCell::new(AccordionWidget::new(rel_path.as_str())));
                let mut overlays = Vec::new();

                for _ in 0..entry.image_entries.len() {
                    let thumbnail_size = {
                        let app_config = APP_CONFIG.lock().map_err(|_| anyhow!("Failed to lock app config"))?;
                        app_config.thumbnail_size
                    } as i32;
                    
                    let fixed_size_container = gtk::Box::new(gtk::Orientation::Vertical, 0);
                    fixed_size_container
                        .set_size_request(thumbnail_size, thumbnail_size);
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

                                for (index, image_entry) in
                                    loaded_entry.image_entries.iter().enumerate()
                                {
                                    if let Some(img) = &image_entry.image {
                                        let mut image_widget = ImageWidget::new();
                                        image_widget
                                            .set_image(&image_entry.image_path, img.clone());

                                        let accordion_widget = accordion_widget.clone();
                                        let overlays = overlays.clone();
                                        glib::MainContext::default().spawn_local(async move {
                                            let overlay = overlays[index].clone();
                                            overlay.add_overlay(image_widget.widget());
                                            accordion_widget.borrow_mut().flow_box.append(&overlay);
                                        });
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

fn open_with_xdg_open(image_path: String) {
    let child = Command::new("xdg-open").arg(image_path).spawn();
    if let Err(e) = child {
        println!("Failed to open image with xdg-open: {e}");
    }
}
