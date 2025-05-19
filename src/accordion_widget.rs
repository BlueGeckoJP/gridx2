use gtk4 as gtk;
use gtk4::prelude::{BoxExt, ObjectExt, WidgetExt};
use gtk4::{Expander, FlowBox, Label, ProgressBar};

use crate::APP_CONFIG;

pub struct AccordionWidget {
    pub widget: gtk::Box,
    pub expander: Expander,
    pub flow_box: FlowBox,
    pub progress_bar: ProgressBar,
}

impl AccordionWidget {
    pub fn new(title: &str) -> Self {
        let expander = Self::create_expander(title);
        let flow_box = Self::create_flow_box();

        expander.set_child(Some(&flow_box));

        let vbox = gtk::Box::new(gtk::Orientation::Vertical, 0);
        vbox.add_css_class("expander-box");

        if let Ok(app_config) = APP_CONFIG.read() {
            match app_config.dark_mode.unwrap_or(true) {
                true => vbox.add_css_class("dark-mode"),
                false => vbox.add_css_class("light-mode"),
            }
        }

        let progress_bar = ProgressBar::new();
        progress_bar.set_visible(false);

        vbox.append(&progress_bar);
        vbox.append(&expander);

        Self {
            widget: vbox,
            expander,
            flow_box,
            progress_bar,
        }
    }

    pub fn connect_expanded<F: Fn(bool) + 'static>(&self, callback: F) {
        self.expander
            .connect_notify_local(Some("expanded"), move |expander, _| {
                let is_expanded = expander.is_expanded();
                callback(is_expanded);
            });
    }

    fn create_flow_box() -> FlowBox {
        let flow_box = FlowBox::new();

        flow_box.set_valign(gtk::Align::Start);
        flow_box.set_max_children_per_line(30);
        flow_box.set_selection_mode(gtk::SelectionMode::None);
        flow_box.set_homogeneous(false);
        flow_box.set_min_children_per_line(1);

        flow_box.set_row_spacing(8);
        flow_box.set_column_spacing(8);

        flow_box
    }

    fn create_expander(title: &str) -> Expander {
        let expander = Expander::new(None);

        let label = Label::new(Some(title));
        label.add_css_class("expander-title");

        expander.set_label_widget(Some(&label));
        expander.set_expanded(false);

        expander
    }
}
