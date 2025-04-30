mod entry;

use gtk4 as gtk;
use gtk4::gio::Cancellable;
use gtk4::prelude::{ActionMapExt, ApplicationExt, ApplicationExtManual, ApplicationWindowExt, BoxExt, Cast, FileExt, GtkApplicationExt, GtkWindowExt};
use gtk4::{gdk, gio, glib, Application, ApplicationWindow, FileDialog, Picture};
use image::GenericImageView;
use std::sync::{Arc, Mutex};

static MAX_DEPTH: u32 = 2;
static THUMBNAIL_SIZE: u32 = 200;

fn main() -> glib::ExitCode {
    let original_dir = Arc::new(Mutex::new(String::from(".")));

    gtk::init().expect("Failed to initialize GTK");

    let app = Application::builder().application_id("me.bluegecko.gridx2").build();

    let d = entry::DirEntry::search(original_dir.lock().unwrap().clone()).unwrap();
    
    app.connect_activate(move |app| {
        let window = ApplicationWindow::builder()
            .application(app)
            .default_width(800)
            .default_height(600)
            .title("gridx2")
            .build();

        let menubar = gio::Menu::new();

        let file_menu = gio::Menu::new();
        file_menu.append(Some("Open Folder"), Some("app.open"));

        menubar.append_submenu(Some("File"), &file_menu);

        app.set_menubar(Some(&menubar));
        window.set_show_menubar(true);

        let original_dir = original_dir.clone();
        let open_action = gio::SimpleAction::new("open", None);
        open_action.connect_activate(glib::clone!(
            #[weak] window,
            move |_, _| {
                let dialog = FileDialog::new();
                let cancellable = Cancellable::new();
                let original_dir = original_dir.clone();
                dialog.select_folder(Some(&window), Some(&cancellable), move |result| {
                    if let Ok(path) = result {
                        if let Some(dir) = path.path() {
                            *original_dir.lock().unwrap() = dir.to_string_lossy().to_string();
                            println!("{:?}", original_dir.lock().unwrap());
                        }
                    }
                });
            }
        ));
        app.add_action(&open_action);

        let vbox = gtk::Box::builder().orientation(gtk::Orientation::Vertical).build();

        d.iter().for_each(|e| {
            let hbox = gtk::Box::builder().orientation(gtk::Orientation::Horizontal).build();
            e.image_entries.iter().for_each(|i| {
                let img = &i.image;
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
                let picture = Picture::for_paintable(&texture);

                hbox.append(&picture);
            });
            vbox.append(&hbox);
        });

        window.set_child(Some(&vbox));
        window.present();
    });
    
    app.run()
}
