use std::{
    num::NonZero,
    sync::{Arc, Mutex, RwLock, RwLockReadGuard},
};

use gtk4::gdk::Texture;
use lru::LruCache;

use crate::state::app_config::AppConfig;

type ImageCache = LruCache<String, Arc<Texture>>;

pub struct Shared {
    config: RwLock<AppConfig>,
    image_cache: Mutex<ImageCache>,
}

impl Shared {
    pub fn new() -> Self {
        Self {
            config: RwLock::new(AppConfig::load().unwrap_or_default()),
            image_cache: Mutex::new(LruCache::new(
                NonZero::new(5000).expect("Failed to create NonZero value"),
            )),
        }
    }

    pub fn config(&self) -> eyre::Result<RwLockReadGuard<'_, AppConfig>> {
        self.config
            .read()
            .map_err(|e| eyre::eyre!("Failed to lock config for read: {}", e))
    }

    pub fn update_config<R>(&self, update_fn: impl FnOnce(&mut AppConfig) -> R) -> eyre::Result<R> {
        let mut config = self
            .config
            .write()
            .map_err(|e| eyre::eyre!("Failed to lock config for write: {}", e))?;
        Ok(update_fn(&mut config))
    }

    pub fn image_cache(&self) -> eyre::Result<std::sync::MutexGuard<'_, ImageCache>> {
        self.image_cache
            .lock()
            .map_err(|e| eyre::eyre!("Failed to lock image cache: {}", e))
    }
}
