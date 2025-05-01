use gtk4 as gtk;
use gtk4::gdk::Texture;
use gtk4::prelude::{BoxExt, TextureExt, WidgetExt};
use gtk4::Picture;
use std::cell::RefCell;
use std::rc::Rc;

pub struct ImageWidget {
    widget: gtk::Box,
    picture: Picture,
    image_path: Rc<RefCell<Option<String>>>,
}

impl ImageWidget {
    pub fn new() -> Self {
        let widget = gtk::Box::new(gtk::Orientation::Vertical, 0);
        widget.set_halign(gtk::Align::Center);
        widget.set_valign(gtk::Align::Center);
        
        let picture = Picture::new();
        picture.set_halign(gtk::Align::Center);
        picture.set_valign(gtk::Align::Center);
        
        widget.append(&picture);
        
        let image_path = Rc::new(RefCell::new(None));
        
        let image_widget = Self {
            widget,
            picture,
            image_path,
        };
        
        image_widget.setup_click_handler();
        
        image_widget
    }
    
    fn setup_click_handler(&self) {
        let image_path_clone = self.image_path.clone();
        
        let click_gesture = gtk::GestureClick::new();
        click_gesture.connect_released(move |_gesture, _n_press, _x, _y| {
            let path = image_path_clone.borrow().clone();
            if let Some(path) = &path { println!("{path}") }
        });
        
        self.picture.add_controller(click_gesture);
    }
    
    pub fn set_image(&mut self, path: &str, texture: Texture) {
        self.picture.set_paintable(Some(&texture));
        self.picture.set_size_request(texture.width(), texture.height());
        self.image_path.replace(Some(path.to_string()));
    }
    
    pub fn widget(&self) -> &gtk::Box {
        &self.widget
    }
}