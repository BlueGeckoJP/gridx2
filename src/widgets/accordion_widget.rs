use std::rc::Rc;

use gtk4::prelude::{BoxExt, ButtonExt, ObjectExt, WidgetExt};
use gtk4::{self as gtk, glib};
use gtk4::{Expander, FlowBox, Label, ProgressBar};

use crate::config::app_config::AppConfig;

pub struct AccordionWidget {
    pub widget: gtk::Box,
    pub expander: Expander,
    pub flow_box: FlowBox,
    pub progress_bar: ProgressBar,
    pub close_button: Rc<gtk::Button>,
}

impl AccordionWidget {
    pub fn new(title: &str, app_config: AppConfig) -> eyre::Result<Self> {
        let expander = Self::create_expander(title);
        let flow_box = Self::create_flow_box();

        expander.set_child(Some(&flow_box));

        let vbox = gtk::Box::new(gtk::Orientation::Vertical, 0);
        vbox.add_css_class("expander-box");

        let config = app_config.get()?;
        match config.dark_mode {
            true => vbox.add_css_class("dark-mode"),
            false => vbox.add_css_class("light-mode"),
        }

        let progress_bar = ProgressBar::new();
        progress_bar.set_visible(false);

        let close_button = gtk::Button::with_label("Close");
        close_button.add_css_class("close-button");
        close_button.connect_clicked(glib::clone!(
            #[weak]
            expander,
            move |_: &gtk4::Button| {
                expander.set_expanded(false);
            }
        ));
        close_button.set_visible(false);

        vbox.append(&progress_bar);
        vbox.append(&expander);
        vbox.append(&close_button);

        Ok(Self {
            widget: vbox,
            expander,
            flow_box,
            progress_bar,
            close_button: Rc::new(close_button),
        })
    }

    pub fn connect_expanded<F: Fn(bool) + 'static>(&self, callback: F) {
        let close_button = self.close_button.clone();
        self.expander
            .connect_notify_local(Some("expanded"), move |expander, _| {
                let is_expanded = expander.is_expanded();
                close_button.set_visible(is_expanded);
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
