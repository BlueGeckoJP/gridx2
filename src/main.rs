mod entry;

use gtk4 as gtk;
use gtk4::{glib, Application, ApplicationWindow};
use gtk4::prelude::{ApplicationExt, ApplicationExtManual, BoxExt, GtkWindowExt};

static MAX_DEPTH: u32 = 2;
static BASE_DIR: &str = ".";

fn main() -> glib::ExitCode {
    gtk::init().expect("Failed to initialize GTK");

    let app = Application::builder().application_id("me.bluegecko.gridx2").build();

    let d = entry::DirEntry::search(BASE_DIR.into()).unwrap();
    println!("{d:?}");
    
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
                hbox.append(&i.image);
            });
            vbox.append(&hbox);
        });

        window.set_child(Some(&vbox));
        window.present();
    });
    
    app.run()
}
