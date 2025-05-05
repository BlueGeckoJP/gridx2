use crate::APP_CONFIG;
use gtk4 as gtk;
use gtk4::prelude::{BoxExt, ButtonExt, EditableExt, GtkWindowExt, WidgetExt};
use gtk4::{glib, Adjustment, ApplicationWindow, SpinButton};

pub struct SettingsWindow {
    window: ApplicationWindow,
}

impl SettingsWindow {
    pub fn new(parent: &ApplicationWindow) -> anyhow::Result<Self> {
        let window = ApplicationWindow::builder()
            .title("Settings")
            .default_width(300)
            .default_height(200)
            .transient_for(parent)
            .modal(true)
            .build();

        let vbox = gtk::Box::new(gtk::Orientation::Vertical, 10);
        vbox.set_margin_top(10);
        vbox.set_margin_bottom(10);
        vbox.set_margin_start(10);
        vbox.set_margin_end(10);
        window.set_child(Some(&vbox));

        let max_depth_box = gtk::Box::new(gtk::Orientation::Horizontal, 10);
        let max_depth_label = gtk::Label::new(Some("Max depth:"));
        let max_depth_spin = SpinButton::new(
            Some(&Adjustment::new(0.0, 1.0, 10.0, 1.0, 5.0, 0.0)),
            1.0,
            0,
        );

        max_depth_box.append(&max_depth_label);
        max_depth_box.append(&max_depth_spin);
        vbox.append(&max_depth_box);

        let thumbnail_box = gtk::Box::new(gtk::Orientation::Horizontal, 10);
        let thumbnail_label = gtk::Label::new(Some("Thumbnail size:"));
        let thumbnail_spin = SpinButton::new(
            Some(&Adjustment::new(0.0, 50.0, 500.0, 10.0, 50.0, 0.0)),
            1.0,
            0,
        );

        thumbnail_box.append(&thumbnail_label);
        thumbnail_box.append(&thumbnail_spin);
        vbox.append(&thumbnail_box);

        let command_box = gtk::Box::new(gtk::Orientation::Horizontal, 10);
        let command_label = gtk::Label::new(Some("Open command:"));
        command_label.set_halign(gtk::Align::Start);

        let command_entry = gtk::Entry::new();

        command_box.append(&command_label);
        command_box.append(&command_entry);
        vbox.append(&command_box);

        let hint_label = gtk::Label::new(Some("Hint: the actual path is assigned to <path>"));
        hint_label.set_halign(gtk::Align::Start);
        vbox.append(&hint_label);

        let button_box = gtk::Box::new(gtk::Orientation::Horizontal, 10);
        let button_save = gtk::Button::with_label("Save");
        let button_cancel = gtk::Button::with_label("Cancel");

        button_box.append(&button_save);
        button_box.append(&button_cancel);
        vbox.append(&button_box);

        let current_config = {
            let config = APP_CONFIG
                .lock()
                .map_err(|_| anyhow::anyhow!("Failed to lock config"))?;
            config.clone()
        };
        max_depth_spin.set_value(current_config.max_depth as f64);
        thumbnail_spin.set_value(current_config.thumbnail_size as f64);
        command_entry.set_text(&current_config.open_command.join(" "));

        button_cancel.connect_clicked(glib::clone!(
            #[weak]
            window,
            move |_| window.close()
        ));

        button_save.connect_clicked(glib::clone!(
            #[weak]
            window,
            #[weak]
            max_depth_spin,
            #[weak]
            thumbnail_spin,
            #[weak]
            command_entry,
            move |_| {
                let mut config = match APP_CONFIG.lock() {
                    Ok(config) => config,
                    Err(_) => return,
                };

                config.max_depth = max_depth_spin.value() as u32;
                config.thumbnail_size = thumbnail_spin.value() as u32;
                config.open_command = command_entry
                    .text()
                    .split_whitespace()
                    .map(|s| s.to_string())
                    .collect();

                if let Err(e) = config.save() {
                    eprintln!("Failed to save config: {e}");
                }

                window.close();
            }
        ));

        Ok(Self { window })
    }

    pub fn show(&self) {
        self.window.present();
    }
}
