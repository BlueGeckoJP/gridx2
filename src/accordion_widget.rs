use gtk4 as gtk;
use gtk4::prelude::{BoxExt, WidgetExt};
use gtk4::{Expander, FlowBox, Label};

pub struct AccordionWidget {
    pub widget: gtk::Box,
    pub flow_box: FlowBox,
}

impl AccordionWidget {
    pub fn new(title: &str) -> Self {
        let expander = Expander::new(None);

        let label = Label::new(Some(title));
        label.add_css_class("expander-title");

        expander.set_label_widget(Some(&label));

        let flow_box = FlowBox::new();

        flow_box.set_valign(gtk::Align::Start);
        flow_box.set_max_children_per_line(30);
        flow_box.set_selection_mode(gtk::SelectionMode::None);
        flow_box.set_homogeneous(false);
        flow_box.set_min_children_per_line(1);

        flow_box.set_row_spacing(8);
        flow_box.set_column_spacing(8);

        expander.set_child(Some(&flow_box));
        expander.set_expanded(false);

        let vbox = gtk::Box::new(gtk::Orientation::Vertical, 0);
        vbox.add_css_class("expander-box");
        vbox.append(&expander);

        Self {
            widget: vbox,
            flow_box,
        }
    }
}
