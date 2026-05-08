use gtk4::{self as gtk, glib, prelude::WidgetExt};
use rayon::iter::{IntoParallelRefMutIterator, ParallelIterator};
use std::{
    cell::RefCell,
    cmp::Ordering,
    rc::Rc,
    sync::{Arc, Mutex, mpsc},
    thread,
    time::Duration,
};

use crate::{
    config::{app_config::AppConfig, raw_config::SortOrder},
    entry,
    file_utils::sort_by_updated_at,
    image_cache::ImageCache,
    image_entry::{ImageEntry, ImageEntryMetrics},
    session::Session,
    utils::natural_sort,
    widgets::{accordion_widget::AccordionWidget, image_widget::ImageWidget},
};

enum LoadMessage {
    Progress(f64),
    Complete(Vec<ImageEntry>),
}

async fn display_loaded_images(
    image_entries: Vec<ImageEntry>,
    accordion_widget: Rc<RefCell<AccordionWidget>>,
    overlays: Vec<gtk::Overlay>,
    app_config: AppConfig,
) {
    let mut sorted_entries = image_entries.clone();

    let config = match app_config.get() {
        Ok(config) => config,
        Err(e) => {
            eprintln!("Failed to get app config: {e}");
            return;
        }
    };

    let sort_order = config.sort_order;
    let descending = config.descending;

    match sort_order {
        SortOrder::Name => sorted_entries.sort_by(|a, b| {
            natural_sort(a.image_path.as_str(), b.image_path.as_str(), descending)
                .unwrap_or(Ordering::Equal)
        }),
        SortOrder::UpdatedAt => sorted_entries.sort_by(|a, b| {
            sort_by_updated_at(a.image_path.as_str(), b.image_path.as_str(), descending)
                .unwrap_or(Ordering::Equal)
        }),
    }

    for (index, image_entry) in sorted_entries.iter().enumerate() {
        if let Some(img) = &image_entry.image {
            let mut image_widget = ImageWidget::new(app_config.clone());
            image_widget.set_image(&image_entry.image_path, img.as_ref());

            let accordion_widget = accordion_widget.clone();
            let overlays = overlays.clone();

            glib::MainContext::default().spawn_local(async move {
                if index < overlays.len() {
                    let overlay = overlays[index].clone();
                    overlay.add_overlay(image_widget.widget());
                    accordion_widget.borrow().flow_box.append(&overlay);
                }
            });

            if index.is_multiple_of(5) {
                glib::timeout_future(Duration::from_millis(10)).await;
            }
        }
    }

    accordion_widget.borrow().progress_bar.set_visible(false);
}

pub async fn load_and_display_images(
    session: Session,
    image_cache: ImageCache,
    app_config: AppConfig,
    accordion_widget: Rc<RefCell<AccordionWidget>>,
    overlays: Vec<gtk::Overlay>,
    index: usize,
) {
    let dir_entry_clone = {
        let dir_entries = session.dir_entries();
        match dir_entries {
            Ok(entries) => {
                if index >= entries.len() {
                    eprintln!("Invalid index: {index}");
                    return;
                }

                entries[index].clone()
            }
            Err(e) => {
                eprintln!("Failed to get dir_entries: {e}");
                return;
            }
        }
    };

    let total_images = dir_entry_clone.image_entries.len();
    let counter = Arc::new(Mutex::new(0f64));

    let (tx, rx) = mpsc::channel::<LoadMessage>();

    let thumbnail_size = match app_config.get() {
        Ok(config) => config.thumbnail_size,
        Err(e) => {
            eprintln!("Failed to get app config: {e}");
            return;
        }
    };

    spawn_image_loading_thread(
        &dir_entry_clone,
        counter,
        total_images,
        tx,
        image_cache,
        thumbnail_size,
    );

    handle_load_messages(accordion_widget, overlays, rx, app_config).await;
}

fn spawn_image_loading_thread(
    loaded_entry_clone: &entry::DirEntry,
    counter: Arc<Mutex<f64>>,
    total_images: usize,
    tx: mpsc::Sender<LoadMessage>,
    image_cache: ImageCache,
    thumbnail_size: u32,
) {
    let mut loaded_entry_clone = loaded_entry_clone.clone();

    let metrics = ImageEntryMetrics::default();

    thread::spawn(move || {
        loaded_entry_clone
            .image_entries
            .par_iter_mut()
            .for_each(|image_entry| {
                if let Err(e) =
                    image_entry.load_image(image_cache.clone(), thumbnail_size, &metrics)
                {
                    eprintln!("Failed to load image: {e}");
                }

                let mut counter = match counter.lock() {
                    Ok(counter) => counter,
                    Err(e) => {
                        eprintln!("Failed to lock counter: {e}");
                        return;
                    }
                };

                *counter += 1.0;

                let progress = *counter / total_images as f64;
                let _ = tx.send(LoadMessage::Progress(progress));
            });

        metrics.show_cache_stats();
        let _ = tx.send(LoadMessage::Complete(
            loaded_entry_clone.image_entries.clone(),
        ));
    });
}

async fn handle_load_messages(
    accordion_widget: Rc<RefCell<AccordionWidget>>,
    overlays: Vec<gtk::Overlay>,
    rx: mpsc::Receiver<LoadMessage>,
    app_config: AppConfig,
) {
    let image_entries = loop {
        match rx.try_recv() {
            Ok(LoadMessage::Progress(progress)) => {
                accordion_widget
                    .borrow()
                    .progress_bar
                    .set_fraction(progress);
                glib::timeout_future(Duration::from_millis(1)).await;
            }
            Ok(LoadMessage::Complete(entries)) => {
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

    display_loaded_images(image_entries, accordion_widget, overlays, app_config).await;
}
