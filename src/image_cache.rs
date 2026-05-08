use std::{
    num::NonZeroUsize,
    sync::{Arc, Mutex},
};

use gtk4::gdk::Texture;
use lru::LruCache;

type CacheKey = (String, (usize, usize)); // (image_path, (width, height))
type CacheValue = Arc<Texture>;

#[derive(Clone)]
pub struct ImageCache {
    inner: Arc<Mutex<LruCache<CacheKey, CacheValue>>>,
}

impl ImageCache {
    pub fn new(capacity: usize) -> Self {
        Self {
            inner: Arc::new(Mutex::new(LruCache::new(
                NonZeroUsize::new(capacity).expect("Capacity must be greater than 0"),
            ))),
        }
    }

    pub fn get(&self, path: String, size: (usize, usize)) -> eyre::Result<Option<Arc<Texture>>> {
        let mut cache = self
            .inner
            .lock()
            .map_err(|e| eyre::eyre!("Failed to lock image cache: {e}"))?;
        Ok(cache.get(&(path, size)).cloned())
    }

    pub fn put(
        &self,
        path: String,
        size: (usize, usize),
        texture: Arc<Texture>,
    ) -> eyre::Result<()> {
        let mut cache = self
            .inner
            .lock()
            .map_err(|e| eyre::eyre!("Failed to lock image cache: {e}"))?;
        cache.put((path, size), texture);
        Ok(())
    }
}
