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
        Ok(default_path)
    }

    pub fn into_raw_config(self, gtk_dark_mode: bool) -> RawConfig {
        let default = RawConfig::default();

        RawConfig {
            max_depth: self.max_depth.unwrap_or(default.max_depth),
            thumbnail_size: self.thumbnail_size.unwrap_or(default.thumbnail_size),
            open_command: self.open_command.unwrap_or(default.open_command),
            dark_mode: self.dark_mode.unwrap_or(gtk_dark_mode),
            sort_order: match self.sort_order.as_deref() {
                Some("name") => SortOrder::Name,
                Some("updated_at") => SortOrder::UpdatedAt,
                _ => default.sort_order,
            },
            descending: self.descending.unwrap_or(default.descending),
        }
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
