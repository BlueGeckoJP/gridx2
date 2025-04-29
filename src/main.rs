mod entry;

use gtk4 as gtk;
use gtk4::{glib, Application, ApplicationWindow};
use gtk4::prelude::{ApplicationExt, ApplicationExtManual, GtkWindowExt};

static MAX_DEPTH: u32 = 2;
static BASE_DIR: &str = ".";

fn main() -> glib::ExitCode {
    gtk::init().expect("Failed to initialize GTK");

    let app = Application::builder().application_id("me.bluegecko.gridx2").build();

    let d = entry::DirEntry::search(BASE_DIR.into()).unwrap();
    println!("{d:?}");
    
    app.connect_activate(|app| {
        let window = ApplicationWindow::builder()
            .application(app)
            .default_width(800)
            .default_height(600)
            .title("gridx2")
            .build();
        window.present();
    });
    
    app.run()
}
