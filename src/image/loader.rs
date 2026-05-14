//! The responsibility: orchestrate thumbnail loading, progress updates, sorting, and presentation.

use std::sync::Arc;

use gtk4::{self as gtk, glib, prelude::WidgetExt};
use std::{
    cell::RefCell,
    rc::Rc,
    sync::mpsc,
    time::Duration,
};

use gtk4::gdk::Texture;
use gtk4::prelude::Cast;

use crate::{
    config::app_config::AppConfig,
    directory::entry::DirEntry,
    image::{
        cache::ImageCache,
        grid_presenter::ImageGridPresenter,
        sorter::ImageSorter,
        thumbnail_loader::{DecodedThumbnail, LoadedImage, ThumbnailLoadMessage, ThumbnailLoader},
    },
    ui::widgets::accordion_widget::AccordionWidget,
};

/// A UI-ready thumbnail whose GTK texture was constructed on the main context.
pub struct PresentableImage {
    pub image_path: String,
    pub texture: Texture,
}

/// Loads thumbnails for one directory and displays the finished grid in its accordion.
///
/// This is the UI orchestration boundary: decoding is delegated to `ThumbnailLoader`, ordering to
/// `ImageSorter`, and GTK rendering to `ImageGridPresenter`.
pub async fn load_and_display_images(
    dir_entry: Arc<DirEntry>,
    image_cache: ImageCache,
    app_config: AppConfig,
    accordion_widget: Rc<RefCell<AccordionWidget>>,
    overlays: Vec<gtk::Overlay>,
) {
    let thumbnail_size = match app_config.get() {
        Ok(config) => config.thumbnail_size,
        Err(e) => {
            eprintln!("Failed to get app config: {e}");
            return;
        }
    };

    let rx = ThumbnailLoader::new(image_cache, thumbnail_size).spawn(dir_entry);
    handle_load_messages(accordion_widget, overlays, rx, app_config).await;
}

/// Polls thumbnail load messages and updates the progress bar until the batch completes.
///
/// Once complete, this applies the configured sort order and hands rendering to the presenter.
async fn handle_load_messages(
    accordion_widget: Rc<RefCell<AccordionWidget>>,
    overlays: Vec<gtk::Overlay>,
    rx: mpsc::Receiver<ThumbnailLoadMessage>,
    app_config: AppConfig,
) {
    let image_entries = loop {
        match rx.try_recv() {
            Ok(ThumbnailLoadMessage::Progress(progress)) => {
                accordion_widget
                    .borrow()
                    .progress_bar
                    .set_fraction(progress);
                glib::timeout_future(Duration::from_millis(1)).await;
            }
            Ok(ThumbnailLoadMessage::Complete(entries)) => {
                accordion_widget.borrow().progress_bar.set_fraction(1.0);
                glib::timeout_future(Duration::from_millis(1)).await;
                break entries;
            }
            Err(mpsc::TryRecvError::Empty) => {
                glib::timeout_future(Duration::from_millis(16)).await;
            }
            Err(mpsc::TryRecvError::Disconnected) => {
                eprintln!("Image loading channel disconnected before completion");
                accordion_widget.borrow().progress_bar.set_visible(false);
                return;
            }
        }
    };

    let config = match app_config.get() {
        Ok(config) => config,
        Err(e) => {
            eprintln!("Failed to get app config: {e}");
            return;
        }
    };

    let image_entries = ImageSorter::sort(image_entries, &config);
    let image_entries = present_images(image_entries);
    ImageGridPresenter::new(accordion_widget, overlays, app_config)
        .display(image_entries)
        .await;
}

/// Converts decoded thumbnail buffers into GTK textures on the main context.
fn present_images(image_entries: Vec<LoadedImage>) -> Vec<PresentableImage> {
    image_entries
        .into_iter()
        .map(|image_entry| PresentableImage {
            image_path: image_entry.image_path,
            texture: decoded_thumbnail_to_texture(&image_entry.thumbnail),
        })
        .collect()
}

fn decoded_thumbnail_to_texture(thumbnail: &Arc<DecodedThumbnail>) -> Texture {
    gtk::gdk::MemoryTexture::new(
        thumbnail.width,
        thumbnail.height,
        gtk::gdk::MemoryFormat::R8g8b8a8,
        &glib::Bytes::from(&thumbnail.pixels),
        thumbnail.stride,
    )
    .upcast::<Texture>()
}
