use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::config::raw_config::{RawConfig, SortOrder};

#[derive(Clone, Serialize, Deserialize)]
pub struct ConfigStore {
    max_depth: Option<u32>,
    thumbnail_size: Option<u32>,
    open_command: Option<Vec<String>>,
    dark_mode: Option<bool>,
    sort_order: Option<String>,
    descending: Option<bool>,
}

impl ConfigStore {
    /// Returns an error if the specified file does not exist.
    /// Loading default values is not included in this function's responsibilities.
    pub fn load(path: &Path) -> eyre::Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let config: Self = toml::from_str(&content)?;
        Ok(config)
    }

    pub fn save(&self, path: &Path) -> eyre::Result<()> {
        let content = toml::to_string(self)?;
        std::fs::write(path, content)?;
        Ok(())
    }

    pub fn get_default_path() -> eyre::Result<PathBuf> {
        let home_path = home::home_dir().ok_or_else(|| eyre::eyre!("No home directory found"))?;
        let default_path = home_path.join(".gridx2.toml");
        Ok(default_path.canonicalize()?)
    }
}

impl From<RawConfig> for ConfigStore {
    fn from(value: RawConfig) -> Self {
        Self {
            max_depth: Some(value.max_depth),
            thumbnail_size: Some(value.thumbnail_size),
            open_command: Some(value.open_command),
            dark_mode: Some(value.dark_mode),
            sort_order: Some(match value.sort_order {
                SortOrder::Name => "name".to_string(),
                SortOrder::UpdatedAt => "updated_at".to_string(),
            }),
            descending: Some(value.descending),
        }
    }
}

impl From<ConfigStore> for RawConfig {
    fn from(value: ConfigStore) -> Self {
        let default = RawConfig::default();

        Self {
            max_depth: value.max_depth.unwrap_or(default.max_depth),
            thumbnail_size: value.thumbnail_size.unwrap_or(default.thumbnail_size),
            open_command: value.open_command.unwrap_or(default.open_command),
            dark_mode: value.dark_mode.unwrap_or(default.dark_mode),
            sort_order: match value.sort_order.as_deref() {
                Some("name") => SortOrder::Name,
                Some("updated_at") => SortOrder::UpdatedAt,
                _ => default.sort_order,
            },
            descending: value.descending.unwrap_or(default.descending),
        }
    }
}
