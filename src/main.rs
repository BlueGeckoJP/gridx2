mod entry;
mod image_widget;

use crate::image_widget::ImageWidget;
use gtk4 as gtk;
use gtk4::gio::Cancellable;
use gtk4::prelude::{ActionMapExt, ApplicationExt, ApplicationExtManual, ApplicationWindowExt, BoxExt, Cast, FileExt, GtkApplicationExt, GtkWindowExt, WidgetExt};
use gtk4::{gdk, gio, glib, Application, ApplicationWindow, FileDialog};
use image::GenericImageView;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::{Arc, Mutex};

static MAX_DEPTH: u32 = 2;
static THUMBNAIL_SIZE: u32 = 200;

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

    let app = Application::builder().application_id("me.bluegecko.gridx2").build();

    app.connect_activate(move |app| {
        build_ui(app);
    });
    
    app.run()
}

fn build_ui(app: &Application) {
    let app_state = Arc::new(Mutex::new(AppState::new()));

    let window = ApplicationWindow::builder()
        .application(app)
        .default_width(800)
        .default_height(600)
        .title("gridx2")
        .build();

    // Build layout
    let vbox = gtk::Box::builder().orientation(gtk::Orientation::Vertical).build();
    window.set_child(Some(&vbox));

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
            #[weak] window,
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
                                update_entry(app_state.clone(), &app_ui.borrow().top_vbox);
                            });
                        }
                    }
                });
            }
        ));
    app.add_action(&open_action);

    // Finalize
    window.present();
}

fn update_entry(app_state: Arc<Mutex<AppState>>, vbox: &gtk::Box) {
    while let Some(child) = vbox.first_child() {
        vbox.remove(&child);
    }

    let app_state_guard = app_state.lock().unwrap();
    let dir_path = app_state_guard.original_dir.clone();
    drop(app_state_guard);

    let entries = entry::DirEntry::search(dir_path);

    match entries {
        Ok(dir_entries) => {
            let mut app_state_guard = app_state.lock().unwrap();
            app_state_guard.dir_entries = dir_entries;
            let dir_entries = &app_state_guard.dir_entries;

            for entry in dir_entries {
                let hbox = gtk::Box::builder().orientation(gtk::Orientation::Horizontal).build();

                for image_entry in &entry.image_entries {
                    let img = &image_entry.image;
                    let rgba_img = img.to_rgba8();
                    let (width, height) = img.dimensions();
                    let bytes = glib::Bytes::from(&rgba_img.into_raw());

                    let texture = gdk::MemoryTexture::new(
                        width as i32,
                        height as i32,
                        gdk::MemoryFormat::R8g8b8a8,
                        &bytes,
                        (4 * width) as usize,
                    ).upcast::<gdk::Texture>();

                    let mut image_widget = ImageWidget::new();
                    image_widget.set_image(&image_entry.image_path, texture);
                    hbox.append(image_widget.widget());
                }
                vbox.append(&hbox);
            }
        },
        Err(e) => {
            println!("Error: {e}");
        }
    }
}