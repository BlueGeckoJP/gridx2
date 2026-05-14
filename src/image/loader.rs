//! The responsibility: orchestrate thumbnail loading, progress updates, sorting, and presentation.

use gtk4::{self as gtk, glib, prelude::WidgetExt};
use std::{cell::RefCell, rc::Rc, sync::mpsc, time::Duration};

use crate::{
    config::app_config::AppConfig,
    directory::entry::DirEntry,
    image::{
        cache::ImageCache,
        grid_presenter::ImageGridPresenter,
        sorter::ImageSorter,
        thumbnail_loader::{ThumbnailLoadMessage, ThumbnailLoader},
    },
    ui::widgets::accordion_widget::AccordionWidget,
};

/// Loads thumbnails for one directory and displays the finished grid in its accordion.
///
/// This is the UI orchestration boundary: decoding is delegated to `ThumbnailLoader`, ordering to
/// `ImageSorter`, and GTK rendering to `ImageGridPresenter`.
pub async fn load_and_display_images(
    dir_entry: DirEntry,
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
                glib::timeout_future(Duration::from_millis(16)).await;
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
    ImageGridPresenter::new(accordion_widget, overlays, app_config)
        .display(image_entries)
        .await;
}
