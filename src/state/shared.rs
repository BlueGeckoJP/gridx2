use std::{
    num::NonZero,
    sync::{Arc, Mutex},
};

use gtk4::gdk::Texture;
use lru::LruCache;

type ImageCache = LruCache<String, Arc<Texture>>;

pub struct Shared {
    image_cache: Mutex<ImageCache>,
}

impl Shared {
    pub fn new() -> Self {
        Self {
            image_cache: Mutex::new(LruCache::new(
                NonZero::new(5000).expect("Failed to create NonZero value"),
            )),
        }
    }

    pub fn image_cache(&self) -> eyre::Result<std::sync::MutexGuard<'_, ImageCache>> {
        self.image_cache
            .lock()
            .map_err(|e| eyre::eyre!("Failed to lock image cache: {}", e))
    }
}
