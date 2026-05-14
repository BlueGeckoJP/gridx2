//! The responsibility: render loaded thumbnails into the accordion image grid.

use std::{cell::RefCell, rc::Rc, time::Duration};

use gtk4::{self as gtk, gdk::Texture, glib, prelude::Cast, prelude::WidgetExt};

use crate::{
    config::app_config::AppConfig,
    image::thumbnail_loader::{DecodedThumbnail, LoadedImage},
    ui::widgets::{accordion_widget::AccordionWidget, image_widget::ImageWidget},
};

/// Presents loaded thumbnails in GTK widgets and wires image click behavior.
///
/// This owns GTK rendering details only; it does not load, cache, or sort image data.
pub struct ImageGridPresenter {
    accordion_widget: Rc<RefCell<AccordionWidget>>,
    overlays: Vec<gtk::Overlay>,
    app_config: AppConfig,
}

impl ImageGridPresenter {
    /// Creates a presenter for a single accordion expansion render pass.
    pub fn new(
        accordion_widget: Rc<RefCell<AccordionWidget>>,
        overlays: Vec<gtk::Overlay>,
        app_config: AppConfig,
    ) -> Self {
        Self {
            accordion_widget,
            overlays,
            app_config,
        }
    }

    /// Adds the loaded image widgets to the flow box in small batches to keep the UI responsive.
    pub async fn display(self, image_entries: Vec<LoadedImage>) {
        for (index, image_entry) in image_entries.into_iter().enumerate() {
            let image_widget = ImageWidget::new();
            let texture = decoded_thumbnail_to_texture(&image_entry.thumbnail);
            image_widget.set_image(&texture);

            let image_path = image_entry.image_path;
            let app_config = self.app_config.clone();

            image_widget.connect_clicked(move || {
                if let Err(e) = crate::image::actions::open_image(&app_config, &image_path) {
                    eprintln!("Failed to open image: {e}");
                }
            });

            let accordion_widget = self.accordion_widget.clone();
            let overlay = self.overlays.get(index).cloned();

            glib::MainContext::default().spawn_local(async move {
                if let Some(overlay) = overlay {
                    overlay.add_overlay(image_widget.widget());
                    accordion_widget.borrow().flow_box.append(&overlay);
                }
            });

            if index.is_multiple_of(5) {
                glib::timeout_future(Duration::from_millis(10)).await;
            }
        }

        self.accordion_widget
            .borrow()
            .progress_bar
            .set_visible(false);
    }
}

fn decoded_thumbnail_to_texture(thumbnail: &DecodedThumbnail) -> Texture {
    gtk::gdk::MemoryTexture::new(
        thumbnail.width,
        thumbnail.height,
        gtk::gdk::MemoryFormat::R8g8b8a8,
        &glib::Bytes::from(&thumbnail.pixels),
        thumbnail.stride,
    )
    .upcast::<Texture>()
}
