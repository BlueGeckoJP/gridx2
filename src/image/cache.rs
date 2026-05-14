//! The responsibility: cache decoded thumbnail pixel buffers by path and thumbnail size.

use std::{
    num::NonZeroUsize,
    sync::{Arc, Mutex},
};

use lru::LruCache;

use crate::image::thumbnail_loader::DecodedThumbnail;

type CacheKey = (String, (usize, usize)); // (image_path, (width, height))
type CacheValue = Arc<DecodedThumbnail>;

/// Shared LRU cache for decoded thumbnail buffers.
///
/// Use this when thumbnail loading needs to avoid re-decoding the same image at the same size.
/// The cache is internally synchronized so background loaders can clone and share it.
#[derive(Clone)]
pub struct ImageCache {
    inner: Arc<Mutex<LruCache<CacheKey, CacheValue>>>,
}

impl ImageCache {
    /// Creates a cache with the maximum number of thumbnail entries to retain.
    pub fn new(capacity: usize) -> Self {
        Self {
            inner: Arc::new(Mutex::new(LruCache::new(
                NonZeroUsize::new(capacity).expect("Capacity must be greater than 0"),
            ))),
        }
    }

    /// Looks up a cached thumbnail by image path and requested thumbnail size.
    pub fn get(
        &self,
        path: String,
        size: (usize, usize),
    ) -> eyre::Result<Option<Arc<DecodedThumbnail>>> {
        let mut cache = self
            .inner
            .lock()
            .map_err(|e| eyre::eyre!("Failed to lock image cache: {e}"))?;
        Ok(cache.get(&(path, size)).cloned())
    }

    /// Stores a decoded thumbnail for later reuse by the thumbnail loader.
    pub fn put(
        &self,
        path: String,
        size: (usize, usize),
        thumbnail: Arc<DecodedThumbnail>,
    ) -> eyre::Result<()> {
        let mut cache = self
            .inner
            .lock()
            .map_err(|e| eyre::eyre!("Failed to lock image cache: {e}"))?;
        cache.put((path, size), thumbnail);
        Ok(())
    }
}
