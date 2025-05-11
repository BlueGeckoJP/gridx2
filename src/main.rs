mod accordion_widget;
mod app_config;
mod entry;
mod image_cache;
mod image_entry;
mod image_widget;
mod settings_window;

use crate::accordion_widget::AccordionWidget;
use crate::app_config::AppConfig;
use crate::image_cache::ImageCache;
use crate::image_entry::ImageEntry;
use crate::image_widget::ImageWidget;
use crate::settings_window::SettingsWindow;
use anyhow::anyhow;
use gtk4 as gtk;
use gtk4::gio::Cancellable;
use gtk4::prelude::{
    ActionMapExt, ApplicationExt, ApplicationExtManual, ApplicationWindowExt, BoxExt, FileExt,
    GtkApplicationExt, GtkWindowExt, WidgetExt,
};
use gtk4::{gdk, gio, glib, Application, ApplicationWindow, CssProvider, FileDialog};
use rayon::prelude::*;
use regex::Regex;
use std::cell::RefCell;
use std::cmp::{min, Ordering};
use std::path::Path;
use std::process::{Command, Stdio};
use std::rc::Rc;
use std::sync::{mpsc, Arc, LazyLock, Mutex};
use std::thread;
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

    // Build actions
    build_action(app, &window, &app_ui, &app_state);

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

fn build_action(
    app: &Application,
    window: &ApplicationWindow,
    app_ui: &Rc<RefCell<AppUI>>,
    app_state: &Arc<Mutex<AppState>>,
) {
    let app_ui = app_ui.clone();
    let app_state = app_state.clone();

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
}

fn update_entry(app_state: Arc<Mutex<AppState>>, vbox: &gtk::Box) -> anyhow::Result<()> {
    clear_ui(vbox);

    let (original_dir, entries_indies) = search_and_prepare_entries(app_state.clone())?;

    for (index, entry) in entries_indies.iter().enumerate() {
        create_blank_accordion_widget(
            vbox,
            entry.image_entries.len(),
            &get_relative_path(&original_dir, &entry.dir_path)?,
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

fn create_blank_accordion_widget(
    vbox: &gtk::Box,
    count: usize,
    title: &str,
    index: usize,
    app_state: Arc<Mutex<AppState>>,
) -> anyhow::Result<()> {
    let accordion_widget = Rc::new(RefCell::new(AccordionWidget::new(title)));
    let mut overlays = Vec::new();

    for _ in 0..count {
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

        accordion_widget.borrow().flow_box.append(&overlay);
        overlays.push(overlay);
    }

    vbox.append(&accordion_widget.borrow().widget);

    setup_accordion_expand_handler(index, accordion_widget, overlays, app_state);

    Ok(())
}

fn setup_accordion_expand_handler(
    index: usize,
    accordion_widget: Rc<RefCell<AccordionWidget>>,
    overlays: Vec<gtk::Overlay>,
    app_state: Arc<Mutex<AppState>>,
) {
    accordion_widget
        .clone()
        .borrow()
        .connect_expanded(move |is_expanded| {
            if is_expanded {
                let app_state_clone = app_state.clone();
                let accordion_widget = accordion_widget.clone();
                let overlays = overlays.clone();

                prepare_accordion_for_loading(&accordion_widget);

                glib::spawn_future_local(async move {
                    load_and_display_images(app_state_clone, accordion_widget, overlays, index)
                        .await;
                });
            }
        });
}

fn prepare_accordion_for_loading(accordion_widget: &Rc<RefCell<AccordionWidget>>) {
    let accordion_widget = accordion_widget.borrow();

    while let Some(child) = accordion_widget.flow_box.first_child() {
        accordion_widget.flow_box.remove(&child);
    }

    accordion_widget.progress_bar.set_fraction(0.0);
    accordion_widget.progress_bar.set_visible(true);
}

async fn load_and_display_images(
    app_state: Arc<Mutex<AppState>>,
    accordion_widget: Rc<RefCell<AccordionWidget>>,
    overlays: Vec<gtk::Overlay>,
    index: usize,
) {
    let dir_entry_clone = {
        match app_state.lock() {
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

    let total_images = dir_entry_clone.image_entries.len();
    let counter = Arc::new(Mutex::new(0f64));

    let (tx, rx) = mpsc::channel::<f64>();
    let (done_tx, done_rx) = mpsc::channel::<Vec<ImageEntry>>();

    let accordion_widget_cloned = accordion_widget.clone();
    let loaded_entry = dir_entry_clone;
    let loaded_entry_clone = loaded_entry.clone();

    spawn_image_loading_thread(&loaded_entry_clone, counter, total_images, tx, done_tx);

    update_progress_bar(accordion_widget_cloned.clone(), rx).await;

    display_loaded_images(done_rx, accordion_widget_cloned, overlays).await;
}

fn spawn_image_loading_thread(
    loaded_entry_clone: &entry::DirEntry,
    counter: Arc<Mutex<f64>>,
    total_images: usize,
    tx: mpsc::Sender<f64>,
    done_tx: mpsc::Sender<Vec<ImageEntry>>,
) {
    let mut loaded_entry_clone = loaded_entry_clone.clone();

    thread::spawn(move || {
        loaded_entry_clone
            .image_entries
            .par_iter_mut()
            .for_each(|image_entry| {
                if let Err(e) = image_entry.load_image() {
                    eprintln!("Failed to load image: {e}");
                }

                let mut counter = match counter.lock() {
                    Ok(counter) => counter,
                    Err(e) => {
                        eprintln!("Failed to lock counter: {e}");
                        return;
                    }
                };

                *counter += 1.0;

                let progress = *counter / total_images as f64;
                let _ = tx.send(progress);
            });

        let _ = tx.send(1.0);
        let _ = done_tx.send(loaded_entry_clone.image_entries.clone());
    });
}

async fn update_progress_bar(
    accordion_widget: Rc<RefCell<AccordionWidget>>,
    rx: mpsc::Receiver<f64>,
) {
    while let Ok(progress) = rx.recv() {
        accordion_widget
            .borrow()
            .progress_bar
            .set_fraction(progress);
        glib::timeout_future(Duration::from_millis(10)).await;
    }
}

async fn display_loaded_images(
    done_rx: mpsc::Receiver<Vec<ImageEntry>>,
    accordion_widget: Rc<RefCell<AccordionWidget>>,
    overlays: Vec<gtk::Overlay>,
) {
    let image_entries = match done_rx.recv() {
        Ok(image_entries) => {
            let mut sorted_entries = image_entries.clone();
            sorted_entries.sort_by(|a, b| {
                natural_sort(a.image_path.as_str(), b.image_path.as_str())
                    .unwrap_or(Ordering::Equal)
            });
            sorted_entries
        }
        Err(e) => {
            eprintln!("Failed to receive image entries: {e}");
            return;
        }
    };

    for (index, image_entry) in image_entries.iter().enumerate() {
        if let Some(img) = &image_entry.image {
            let mut image_widget = ImageWidget::new();
            image_widget.set_image(&image_entry.image_path, img.clone());

            let accordion_widget = accordion_widget.clone();
            let overlays = overlays.clone();

            glib::MainContext::default().spawn_local(async move {
                if index < overlays.len() {
                    let overlay = overlays[index].clone();
                    overlay.add_overlay(image_widget.widget());
                    accordion_widget.borrow().flow_box.append(&overlay);
                }
            });

            if index % 5 == 0 {
                glib::timeout_future(Duration::from_millis(10)).await;
            }
        }
    }

    accordion_widget.borrow().progress_bar.set_visible(false);
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

    cmd.stdout(Stdio::null())
        .stderr(Stdio::null())
        .stdin(Stdio::null());

    cmd.spawn()?;

    Ok(())
}

fn natural_sort(a: &str, b: &str) -> anyhow::Result<Ordering> {
    let re_all = Regex::new(r"(\d+)|(\D+)")?;
    let re_num = Regex::new(r"^\d+$")?;

    let a_parts: Vec<&str> = re_all.find_iter(a).map(|m| m.as_str()).collect();
    let b_parts: Vec<&str> = re_all.find_iter(b).map(|m| m.as_str()).collect();

    for i in 0..min(a_parts.len(), b_parts.len()) {
        let a_part = a_parts[i];
        let b_part = b_parts[i];

        if a_part == b_part {
            continue;
        }

        let order = if re_num.is_match(a_part) && re_num.is_match(b_part) {
            let a_num = a_part.parse::<isize>().unwrap_or(0);
            let b_num = b_part.parse::<isize>().unwrap_or(0);
            a_num.cmp(&b_num)
        } else if re_num.is_match(a_part) {
            Ordering::Less
        } else if re_num.is_match(b_part) {
            Ordering::Greater
        } else {
            a_part.cmp(b_part)
        };

        if order != Ordering::Equal {
            return Ok(order);
        }
    }

    Ok(a_parts.len().cmp(&b_parts.len()))
}
