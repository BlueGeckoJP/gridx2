mod entry;

use gtk4 as gtk;
use gtk4::prelude::{ApplicationExt, ApplicationExtManual, BoxExt, Cast, GtkWindowExt};
use gtk4::{gdk, glib, Application, ApplicationWindow, Picture};
use image::GenericImageView;

static MAX_DEPTH: u32 = 2;
static BASE_DIR: &str = ".";
static THUMBNAIL_SIZE: u32 = 200;

fn main() -> glib::ExitCode {
    gtk::init().expect("Failed to initialize GTK");

    let app = Application::builder().application_id("me.bluegecko.gridx2").build();

    let d = entry::DirEntry::search(BASE_DIR.into()).unwrap();
    
    app.connect_activate(move |app| {
        let window = ApplicationWindow::builder()
            .application(app)
            .default_width(800)
            .default_height(600)
            .title("gridx2")
            .build();

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
