use crate::config::raw_config::{RawConfig, SORT_ORDER_VARIANTS};
use gtk4::glib::object::Cast;
use gtk4::prelude::{BoxExt, ButtonExt, EditableExt, GtkWindowExt, WidgetExt};
use gtk4::{self as gtk, gio::ListStore};
use gtk4::{Adjustment, ApplicationWindow, DropDown, SpinButton, glib};

pub struct SettingsWindow {
    window: ApplicationWindow,
}

impl SettingsWindow {
    pub fn new<F: Fn(RawConfig) + 'static>(
        parent: &ApplicationWindow,
        current_config: RawConfig,
        on_save: F,
    ) -> eyre::Result<Self> {
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

        let sort_order_model = ListStore::new::<gtk::StringObject>();
        for order in SORT_ORDER_VARIANTS {
            let item = gtk::StringObject::new(order);
            sort_order_model.append(&item);
        }

        let sort_order_box = gtk::Box::new(gtk::Orientation::Horizontal, 10);
        let sort_order_label = gtk::Label::new(Some("Sort order:"));
        sort_order_label.set_halign(gtk::Align::Start);

        let sort_order_dropdown = DropDown::from_strings(SORT_ORDER_VARIANTS);
        sort_order_dropdown.set_hexpand(true);

        sort_order_box.append(&sort_order_label);
        sort_order_box.append(&sort_order_dropdown);
        vbox.append(&sort_order_box);

        let descending_box = gtk::Box::new(gtk::Orientation::Horizontal, 10);
        let descending_label = gtk::Label::new(Some("Descending Order:"));
        let descending_switch = gtk::Switch::new();

        descending_box.append(&descending_label);
        descending_box.append(&descending_switch);
        vbox.append(&descending_box);

        let button_box = gtk::Box::new(gtk::Orientation::Horizontal, 10);
        let button_save = gtk::Button::with_label("Save");
        let button_cancel = gtk::Button::with_label("Cancel");

        button_box.append(&button_save);
        button_box.append(&button_cancel);
        vbox.append(&button_box);

        max_depth_spin.set_value(current_config.max_depth as f64);
        thumbnail_spin.set_value(current_config.thumbnail_size as f64);
        command_entry.set_text(&current_config.open_command.join(" "));
        sort_order_dropdown.set_selected({
            let order_str = current_config.sort_order.to_string();
            SORT_ORDER_VARIANTS
                .iter()
                .position(|&variant| variant == order_str)
                .unwrap_or(0) as u32
        });
        descending_switch.set_active(current_config.descending);

        button_cancel.connect_clicked(glib::clone!(
            #[weak]
            window,
            move |_| window.close()
        ));

        button_save.connect_clicked(glib::clone!(
            #[weak]
            window,
            move |_| {
                let config = RawConfig {
                    max_depth: max_depth_spin.value() as u32,
                    thumbnail_size: thumbnail_spin.value() as u32,
                    open_command: command_entry
                        .text()
                        .split_whitespace()
                        .map(|s| s.to_string())
                        .collect(),
                    sort_order: sort_order_dropdown
                        .selected_item()
                        .and_then(|item| {
                            item.downcast_ref::<gtk::StringObject>()
                                .and_then(|s| s.string().parse().ok())
                        })
                        .unwrap_or(current_config.sort_order.clone()),
                    descending: descending_switch.is_active(),
                    dark_mode: current_config.dark_mode,
                };

                on_save(config);

                window.close();
            }
        ));

        Ok(Self { window })
    }

    pub fn show(&self) {
        self.window.present();
    }
}
