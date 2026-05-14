//! The responsibility: load, resize, cache, and report progress for thumbnail textures.

use std::{
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
        mpsc,
    },
    thread,
    time::Instant,
};

use gtk4::gdk::Texture;
use gtk4::prelude::Cast;
use gtk4::{gdk, glib};
use image::imageops::FilterType;
use image::{GenericImageView, ImageReader};
use rayon::iter::{IntoParallelIterator, ParallelIterator};

use crate::{directory::entry::DirEntry, image::cache::ImageCache, image::entry::ImageEntry};

/// A thumbnail that has been decoded and converted into a GTK texture for presentation.
///
/// This is the boundary object between background thumbnail loading and UI rendering.
#[derive(Clone)]
pub struct LoadedImage {
    pub image_path: String,
    pub texture: Arc<Texture>,
}

/// Progress messages emitted by the thumbnail loading worker.
///
/// Consumers should treat `Complete` as the final message and render the returned images.
pub enum ThumbnailLoadMessage {
    Progress(f64),
    Complete(Vec<LoadedImage>),
}

/// Loads thumbnails on a background thread and reports progress through a channel.
///
/// Use this from UI orchestration code that needs async progress without embedding decode/cache
/// behavior in GTK widget code.
pub struct ThumbnailLoader {
    image_cache: ImageCache,
    thumbnail_size: u32,
}

impl ThumbnailLoader {
    /// Creates a loader with the cache and target square thumbnail size to use for every image.
    pub fn new(image_cache: ImageCache, thumbnail_size: u32) -> Self {
        Self {
            image_cache,
            thumbnail_size,
        }
    }

    /// Starts the load worker and returns the receiving side of the progress channel.
    pub fn spawn(self, dir_entry: Arc<DirEntry>) -> mpsc::Receiver<ThumbnailLoadMessage> {
        let (tx, rx) = mpsc::channel::<ThumbnailLoadMessage>();
        let total_images = dir_entry.image_entries.len();
        let metrics = ThumbnailLoadMetrics::default();
        let image_cache = self.image_cache;
        let thumbnail_size = self.thumbnail_size;
        let counter = Arc::new(AtomicUsize::new(0));

        thread::spawn(move || {
            let loaded_images = dir_entry
                .image_entries
                .clone()
                .into_par_iter()
                .filter_map(|image_entry| {
                    let result =
                        load_thumbnail(&image_entry, image_cache.clone(), thumbnail_size, &metrics);
                    let finished = counter.fetch_add(1, Ordering::Relaxed) + 1;
                    let progress = finished as f64 / total_images as f64;
                    let _ = tx.send(ThumbnailLoadMessage::Progress(progress));

                    match result {
                        Ok(image) => Some(image),
                        Err(e) => {
                            eprintln!("Failed to load image: {e}");
                            None
                        }
                    }
                })
                .collect();

            metrics.show_cache_stats();
            let _ = tx.send(ThumbnailLoadMessage::Complete(loaded_images));
        });

        rx
    }
}

/// Captures cache and disk timings for a single loading batch.
#[derive(Default)]
struct ThumbnailLoadMetrics {
    cache_hits: AtomicUsize,
    cache_misses: AtomicUsize,
    disk_load_time_ms: AtomicUsize,
    cache_access_time_ns: AtomicUsize,
}

impl ThumbnailLoadMetrics {
    /// Prints lightweight diagnostics for manual performance checks.
    fn show_cache_stats(&self) {
        let hits = self.cache_hits.load(Ordering::Relaxed);
        let misses = self.cache_misses.load(Ordering::Relaxed);
        let total = hits + misses;

        if total == 0 {
            println!("No cache accesses recorded.");
            return;
        }

        let hits_percent = (hits as f64 / total as f64) * 100.0;

        let avg_disk_time = if misses > 0 {
            self.disk_load_time_ms.load(Ordering::Relaxed) as f64 / misses as f64
        } else {
            0.0
        };

        let cache_time_ns = self.cache_access_time_ns.load(Ordering::Relaxed);
        let avg_cache_time_ns = cache_time_ns as f64 / total as f64;
        let avg_cache_time_ms = avg_cache_time_ns / 1_000_000.0;

        println!("\nCache stats:");
        println!("Total accesses: {total}");
        println!("Cache hits: {hits} ({hits_percent:.2}%)");
        println!("Cache misses: {misses}");
        println!("Average disk read time: {avg_disk_time:.2}ms");
        println!(
            "Average cache access time: {avg_cache_time_ms:.2}ms (total {avg_cache_time_ns:.2}ns)"
        );
    }
}

/// Loads one thumbnail, preferring the shared LRU cache before decoding from disk.
fn load_thumbnail(
    image_entry: &ImageEntry,
    image_cache: ImageCache,
    thumbnail_size: u32,
    metrics: &ThumbnailLoadMetrics,
) -> eyre::Result<LoadedImage> {
    let cache_start = Instant::now();
    let cache_hit = image_cache.get(
        image_entry.image_path.clone(),
        (thumbnail_size as usize, thumbnail_size as usize),
    )?;

    let cache_time = cache_start.elapsed().as_nanos() as usize;
    metrics
        .cache_access_time_ns
        .fetch_add(cache_time, Ordering::Relaxed);

    if let Some(texture) = cache_hit {
        metrics.cache_hits.fetch_add(1, Ordering::Relaxed);
        return Ok(LoadedImage {
            image_path: image_entry.image_path.clone(),
            texture,
        });
    }

    metrics.cache_misses.fetch_add(1, Ordering::Relaxed);
    let disk_start = Instant::now();
    let texture = load_and_resize_image(&image_entry.image_path, thumbnail_size)?;
    let disk_time = disk_start.elapsed().as_millis() as usize;
    metrics
        .disk_load_time_ms
        .fetch_add(disk_time, Ordering::Relaxed);

    let texture = Arc::new(texture);

    if let Err(e) = image_cache.put(
        image_entry.image_path.clone(),
        (thumbnail_size as usize, thumbnail_size as usize),
        texture.clone(),
    ) {
        eprintln!("Failed to update image cache: {e}");
    }

    Ok(LoadedImage {
        image_path: image_entry.image_path.clone(),
        texture,
    })
}

/// Decodes an image file, resizes it to fit the thumbnail box, and creates a GTK texture.
fn load_and_resize_image(path: &str, thumbnail_size: u32) -> eyre::Result<Texture> {
    let img = ImageReader::open(path)?.decode()?;
    let (width, height) = img.dimensions();
    let (rw, rh) = calculate_size(width, height, thumbnail_size);
    let resized = img.resize(rw, rh, FilterType::Triangle);
    let rgba = resized.to_rgba8();
    let (width, height) = rgba.dimensions();

    let texture = gdk::MemoryTexture::new(
        width as i32,
        height as i32,
        gdk::MemoryFormat::R8g8b8a8,
        &glib::Bytes::from(&rgba.into_raw()),
        (4 * width) as usize,
    )
    .upcast::<Texture>();

    Ok(texture)
}

/// Calculates a proportional size that fits within a square thumbnail target.
fn calculate_size(mut width: u32, mut height: u32, to: u32) -> (u32, u32) {
    match width > height {
        true => {
            height = (height * to) / width;
            width = to;
            (width, height)
        }
        false => {
            width = (width * to) / height;
            height = to;
            (width, height)
        }
    }
}
