//! The responsibility: expose user-triggered image actions.

use crate::{config::app_config::AppConfig, image::external_opener::ExternalOpener};

/// Opens an image path using the configured external opener.
///
/// Use this from UI click handlers so command resolution stays outside widget code.
pub fn open_image(app_config: &AppConfig, image_path: &str) -> eyre::Result<()> {
    let config = app_config.get()?;
    ExternalOpener::new(config.open_command).open(image_path)
}
