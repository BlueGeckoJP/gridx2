use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

static SAVE_DIR: [&str; 1] = ["~/.gridx2.toml"];

#[derive(Serialize, Deserialize)]
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
        let found = SAVE_DIR.iter().find(|dir| Path::new(dir).exists());

        let path = match found {
            Some(dir) => dir,
            None => return Err(anyhow::anyhow!("No config save directory found")),
        };

        let content = fs::read_to_string(path)?;
        let config: Self = toml::from_str(&content)?;

        Ok(config)
    }

    pub fn save(&self) -> anyhow::Result<()> {
        let found = SAVE_DIR.iter().find(|dir| Path::new(dir).exists());

        let path = match found {
            Some(dir) => dir,
            None => return Err(anyhow::anyhow!("No config save directory found")),
        };

        let content = toml::to_string(self)?;
        fs::write(path, content)?;
        Ok(())
    }
}
