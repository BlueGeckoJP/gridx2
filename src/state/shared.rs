use std::{
    num::NonZero,
    sync::{Arc, Mutex, RwLock, RwLockReadGuard},
};

use gtk4::gdk::Texture;
use lru::LruCache;

use crate::{
    errors::{AppError, AppResult},
    state::app_config::AppConfig,
};

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

    pub fn config(&self) -> AppResult<RwLockReadGuard<'_, AppConfig>> {
        self.config
            .read()
            .map_err(|e| AppError::StateLock(format!("Failed to lock config for read: {}", e)))
    }

    pub fn update_config<F>(&self, update_fn: F) -> AppResult<()>
    where
        F: FnOnce(&mut AppConfig),
    {
        let mut config = self
            .config
            .write()
            .map_err(|e| AppError::StateLock(format!("Failed to lock config for write: {}", e)))?;
        update_fn(&mut config);
        Ok(())
    }

    pub fn image_cache(&self) -> AppResult<std::sync::MutexGuard<'_, ImageCache>> {
        self.image_cache
            .lock()
            .map_err(|e| AppError::StateLock(format!("Failed to lock image cache: {}", e)))
    }
}
