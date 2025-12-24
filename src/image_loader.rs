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

enum LoadMessage {
    Progress(f64),
    Complete(Vec<ImageEntry>),
}

async fn display_loaded_images(
    image_entries: Vec<ImageEntry>,
    accordion_widget: Rc<RefCell<AccordionWidget>>,
    overlays: Vec<gtk::Overlay>,
    app_state: Arc<AppState>,
) {
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

    for (index, image_entry) in sorted_entries.iter().enumerate() {
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
        let dir_entries = app_state.with_runtime_ctx(|ctx| ctx.dir_entries.clone());
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

    spawn_image_loading_thread(
        &dir_entry_clone,
        counter,
        total_images,
        tx,
        app_state.clone(),
    );

    handle_load_messages(accordion_widget, overlays, rx, app_state).await;
}

fn spawn_image_loading_thread(
    loaded_entry_clone: &entry::DirEntry,
    counter: Arc<Mutex<f64>>,
    total_images: usize,
    tx: mpsc::Sender<LoadMessage>,
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
    app_state: Arc<AppState>,
) {
    let mut image_entries = vec![];

    while let Ok(msg) = rx.recv() {
        match msg {
            LoadMessage::Progress(progress) => {
                accordion_widget
                    .borrow()
                    .progress_bar
                    .set_fraction(progress);
                glib::timeout_future(Duration::from_millis(1)).await;
            }
            LoadMessage::Complete(entries) => {
                accordion_widget.borrow().progress_bar.set_fraction(1.0);
                glib::timeout_future(Duration::from_millis(1)).await;
                image_entries = entries;
                break;
            }
        }
    }

    display_loaded_images(image_entries, accordion_widget, overlays, app_state).await;
}
