use crate::state::app_state::AppState;
use gtk4::gdk::Texture;
use gtk4::prelude::Cast;
use gtk4::{gdk, glib};
use image::imageops::FilterType;
use image::{GenericImageView, ImageReader};
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Instant;

#[derive(Default)]
pub struct ImageEntryMetrics {
    cache_hits: AtomicUsize,
    cache_misses: AtomicUsize,
    disk_load_time_ms: AtomicUsize,
    cache_access_time_ns: AtomicUsize,
}

impl ImageEntryMetrics {
    pub fn show_cache_stats(&self) {
        let hits = self.cache_hits.load(Ordering::Relaxed);
        let misses = self.cache_misses.load(Ordering::Relaxed);
        let total = hits + misses;

        if total > 0 {
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
}

#[derive(Debug, Clone)]
pub struct ImageEntry {
    pub image_path: String,
    pub image: Option<Arc<Texture>>,
}

impl ImageEntry {
    pub fn load_image(
        &mut self,
        app_state: Arc<AppState>,
        metrics: &ImageEntryMetrics,
    ) -> eyre::Result<()> {
        if self.image.is_some() {
            return Ok(());
        }

        let thumbnail_size = {
            let app_config = app_state.shared.config()?;
            app_config.thumbnail_size
        };

        let cache_start = Instant::now();
        let cache_hit = {
            let mut image_cache = app_state.shared.image_cache()?;
            image_cache.get(&self.image_path).cloned()
        };

        let cache_time = cache_start.elapsed().as_nanos() as usize;
        metrics
            .cache_access_time_ns
            .fetch_add(cache_time, Ordering::Relaxed);

        if let Some(texture) = cache_hit {
            metrics.cache_hits.fetch_add(1, Ordering::Relaxed);
            //println!("Cache hit: {}ns - {}", cache_time, self.image_path);
            self.image = Some(texture);
            return Ok(());
        }

        metrics.cache_misses.fetch_add(1, Ordering::Relaxed);
        let disk_start = Instant::now();

        if let Ok(texture) = self.load_and_resize_image(thumbnail_size) {
            let disk_time = disk_start.elapsed().as_millis() as usize;
            metrics
                .disk_load_time_ms
                .fetch_add(disk_time, Ordering::Relaxed);
            //println!("Disk load: {}ms - {}", disk_time, self.image_path);

            let texture = Arc::new(texture);
            self.image = Some(texture.clone());

            if let Ok(mut image_cache) = app_state.shared.image_cache() {
                image_cache.put(self.image_path.clone(), texture);
            }
        }

        Ok(())
    }

    fn load_and_resize_image(&self, thumbnail_size: u32) -> eyre::Result<Texture> {
        let path = &self.image_path;
        let img = ImageReader::open(path)?.decode()?;
        let (width, height) = img.dimensions();
        let (rw, rh) = self.calculate_size(width, height, thumbnail_size);
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

    fn calculate_size(&self, mut width: u32, mut height: u32, to: u32) -> (u32, u32) {
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
}
