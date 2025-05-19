use gtk4 as gtk;
use gtk4::glib::object::ObjectExt;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Clone)]
pub struct AppConfig {
    pub max_depth: u32,
    pub thumbnail_size: u32,
    pub open_command: Vec<String>,
    pub dark_mode: Option<bool>,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            max_depth: 2,
            thumbnail_size: 200,
            open_command: vec!["xdg-open".into(), "<path>".into()], // the actual path is assigned to <path>
            dark_mode: Some(true),
        }
    }
}

impl AppConfig {
    pub fn load() -> anyhow::Result<Self> {
        let found = Self::get_exist_path()?;

        let content = fs::read_to_string(found)?;
        let mut config: Self = toml::from_str(&content)?;

        if config.dark_mode.is_none() {
            config.dark_mode = Some(Self::get_dark_mode());
        }

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

    fn get_dark_mode() -> bool {
        match gtk::Settings::default() {
            Some(settings) => {
                let theme_name = settings.property::<String>("gtk-theme-name");
                let is_dark_theme = theme_name.to_lowercase().contains("dark");

                let prefer_dark = settings.property::<bool>("gtk-application-prefer-dark-theme");

                is_dark_theme || prefer_dark
            }
            None => {
                println!("No GTK settings found, defaulting to light mode");
                false
            }
        }
    }
}
