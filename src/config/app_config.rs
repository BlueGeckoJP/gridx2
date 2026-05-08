use std::sync::{Arc, RwLock};

use crate::config::{config_store::ConfigStore, raw_config::RawConfig};

#[derive(Clone)]
pub struct AppConfig {
    inner: Arc<RwLock<RawConfig>>,
}

impl AppConfig {
    pub fn init() -> Self {
        let default_path = match ConfigStore::get_default_path() {
            Ok(path) => path,
            Err(e) => {
                eprintln!(
                    "Failed to get default config path: {}. Using default config values.",
                    e
                );
                return Self {
                    inner: Arc::new(RwLock::new(RawConfig::default())),
                };
            }
        };

        let config_store = match ConfigStore::load(&default_path) {
            Ok(store) => store,
            Err(e) => {
                eprintln!(
                    "Failed to load config from {}: {}. Using default config values.",
                    default_path.display(),
                    e
                );
                return Self {
                    inner: Arc::new(RwLock::new(RawConfig::default())),
                };
            }
        };

        let raw_config: RawConfig = config_store.into();

        Self {
            inner: Arc::new(RwLock::new(raw_config)),
        }
    }

    pub fn get(&self) -> eyre::Result<RawConfig> {
        Ok(self.inner.read().map_err(|e| eyre::eyre!("{e}"))?.clone())
    }

    pub fn update(&mut self, new_config: RawConfig) -> eyre::Result<()> {
        *self.inner.write().map_err(|e| eyre::eyre!("{e}"))? = new_config;
        let config_store: ConfigStore = self
            .inner
            .read()
            .map_err(|e| eyre::eyre!("{e}"))?
            .clone()
            .into();
        let default_path = ConfigStore::get_default_path()?;
        config_store.save(&default_path)?;
        Ok(())
    }
}
