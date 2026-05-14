//! The responsibility: cache GTK image textures by path and thumbnail size.

use std::{
    num::NonZeroUsize,
    sync::{Arc, Mutex},
};

use gtk4::gdk::Texture;
use lru::LruCache;

type CacheKey = (String, (usize, usize)); // (image_path, (width, height))
type CacheValue = Arc<Texture>;

/// Shared LRU cache for decoded thumbnail textures.
///
/// Use this when thumbnail loading needs to avoid re-decoding the same image at the same size.
/// The cache is internally synchronized so background loaders can clone and share it.
#[derive(Clone)]
pub struct ImageCache {
    inner: Arc<Mutex<LruCache<CacheKey, CacheValue>>>,
}

impl ImageCache {
    /// Creates a cache with the maximum number of texture entries to retain.
    pub fn new(capacity: usize) -> Self {
        Self {
            inner: Arc::new(Mutex::new(LruCache::new(
                NonZeroUsize::new(capacity).expect("Capacity must be greater than 0"),
            ))),
        }
    }

    /// Looks up a cached texture by image path and requested thumbnail size.
    pub fn get(&self, path: String, size: (usize, usize)) -> eyre::Result<Option<Arc<Texture>>> {
        let mut cache = self
            .inner
            .lock()
            .map_err(|e| eyre::eyre!("Failed to lock image cache: {e}"))?;
        Ok(cache.get(&(path, size)).cloned())
    }

    /// Stores a decoded texture for later reuse by the thumbnail loader.
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
