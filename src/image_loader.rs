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
    entry,
    file_utils::sort_by_updated_at,
    image_entry::{ImageEntry, ImageEntryMetrics},
    state::app_config::SortOrder,
    state::app_state::AppState,
    utils::natural_sort,
    widgets::accordion_widget::AccordionWidget,
    widgets::image_widget::ImageWidget,
};

async fn display_loaded_images(
    done_rx: mpsc::Receiver<Vec<ImageEntry>>,
    accordion_widget: Rc<RefCell<AccordionWidget>>,
    overlays: Vec<gtk::Overlay>,
    app_state: Arc<AppState>,
) {
    let image_entries = match done_rx.recv() {
        Ok(image_entries) => {
            let mut sorted_entries = image_entries.clone();

            let defaults = (SortOrder::Name, false);
            let (sort_order, descending) = app_state.shared.config().map_or(defaults, |cfg| {
                (
                    cfg.sort_order.unwrap_or(defaults.0),
                    cfg.descending.unwrap_or(defaults.1),
                )
            });

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

            sorted_entries
        }
        Err(e) => {
            eprintln!("Failed to receive image entries: {e}");
            return;
        }
    };

    for (index, image_entry) in image_entries.iter().enumerate() {
        if let Some(img) = &image_entry.image {
            let mut image_widget = ImageWidget::new(app_state.clone());
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
    app_state: Arc<AppState>,
    accordion_widget: Rc<RefCell<AccordionWidget>>,
    overlays: Vec<gtk::Overlay>,
    index: usize,
) {
    let dir_entry_clone = {
        match app_state.dir_entries() {
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

    let (tx, rx) = mpsc::channel::<f64>();
    let (done_tx, done_rx) = mpsc::channel::<Vec<ImageEntry>>();
    let (done_tx_check, done_rx_check) = mpsc::channel::<u8>();

    let accordion_widget_cloned = accordion_widget.clone();
    let loaded_entry = dir_entry_clone;
    let loaded_entry_clone = loaded_entry.clone();

    spawn_image_loading_thread(
        &loaded_entry_clone,
        counter,
        total_images,
        tx,
        done_tx,
        done_tx_check,
        app_state.clone(),
    );

    update_progress_bar(accordion_widget_cloned.clone(), rx, done_rx_check).await;

    display_loaded_images(done_rx, accordion_widget_cloned, overlays, app_state).await;
}

fn spawn_image_loading_thread(
    loaded_entry_clone: &entry::DirEntry,
    counter: Arc<Mutex<f64>>,
    total_images: usize,
    tx: mpsc::Sender<f64>,
    done_tx: mpsc::Sender<Vec<ImageEntry>>,
    done_tx_check: mpsc::Sender<u8>,
    app_state: Arc<AppState>,
) {
    let mut loaded_entry_clone = loaded_entry_clone.clone();

    let metrics = ImageEntryMetrics::default();

    thread::spawn(move || {
        loaded_entry_clone
            .image_entries
            .par_iter_mut()
            .for_each(|image_entry| {
                if let Err(e) = image_entry.load_image(app_state.clone(), &metrics) {
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
                let _ = tx.send(progress);
            });

        metrics.show_cache_stats();
        let _ = done_tx.send(loaded_entry_clone.image_entries.clone());
        let _ = done_tx_check.send(0);
    });
}

async fn update_progress_bar(
    accordion_widget: Rc<RefCell<AccordionWidget>>,
    rx: mpsc::Receiver<f64>,
    done_rx_check: mpsc::Receiver<u8>,
) {
    while let Ok(progress) = rx.recv() {
        if done_rx_check.try_recv().is_ok() {
            accordion_widget.borrow().progress_bar.set_fraction(1.0);
            break;
        }
        accordion_widget
            .borrow()
            .progress_bar
            .set_fraction(progress);
        glib::timeout_future(Duration::from_millis(1)).await;
    }
}
