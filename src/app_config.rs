use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Clone)]
pub struct AppConfig {
    pub max_depth: u32,
    pub thumbnail_size: u32,
    pub open_command: Vec<String>,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            max_depth: 2,
            thumbnail_size: 200,
            open_command: vec!["xdg-open".into(), "<path>".into()], // the actual path is assigned to <path>
        }
    }
}

impl AppConfig {
    pub fn load() -> anyhow::Result<Self> {
        let found = Self::get_exist_path()?;

        let content = fs::read_to_string(found)?;
        let config: Self = toml::from_str(&content)?;

        Ok(config)
    }

    pub fn save(&self) -> anyhow::Result<()> {
        let found = Self::get_exist_path()?;

        let content = toml::to_string(self)?;
        fs::write(found, content)?;

        Ok(())
    }

    fn get_exist_path() -> anyhow::Result<PathBuf> {
        let home_path = home::home_dir().ok_or(anyhow::anyhow!("No home directory found"))?;
        let save_path = home_path.join(".gridx2.toml");

        let path = save_path.canonicalize()?;
        if path.exists() {
            return Ok(path);
        }
        Err(anyhow::anyhow!("No config save directory found"))
    }
}
