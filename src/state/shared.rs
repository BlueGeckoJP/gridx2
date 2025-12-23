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

    pub fn config(&self) -> Result<RwLockReadGuard<'_, AppConfig>, String> {
        self.config
            .read()
            .map_err(|e| format!("Failed to lock config: {}", e))
    }

    pub fn update_config<F>(&self, update_fn: F) -> Result<(), String>
    where
        F: FnOnce(&mut AppConfig),
    {
        let mut config = self
            .config
            .write()
            .map_err(|e| format!("Failed to lock config for write: {}", e))?;
        update_fn(&mut config);
        Ok(())
    }

    pub fn image_cache(&self) -> Result<std::sync::MutexGuard<'_, ImageCache>, String> {
        self.image_cache
            .lock()
            .map_err(|e| format!("Failed to lock image cache: {}", e))
    }
}
