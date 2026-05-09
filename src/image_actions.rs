use crate::{config::app_config::AppConfig, file_utils::open_with_xdg_open};

pub fn open_image(app_config: &AppConfig, image_path: &str) -> eyre::Result<()> {
    let config = app_config.get()?;
    open_with_xdg_open(image_path.to_string(), config.open_command)
}
