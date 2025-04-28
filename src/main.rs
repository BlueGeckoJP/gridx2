use gtk4 as gtk;
use gtk4::{glib, Application, ApplicationWindow};
use gtk4::prelude::{ApplicationExt, ApplicationExtManual, GtkWindowExt};

fn main() -> glib::ExitCode {
    let app = Application::builder().application_id("me.bluegecko.gridx2").build();
    
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