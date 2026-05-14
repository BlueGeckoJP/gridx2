//! The responsibility: expose user-triggered image actions.

use gtk4::{self as gtk, prelude::*};

use crate::{config::app_config::AppConfig, image::external_opener::ExternalOpener};

/// Opens an image path using the configured external opener.
///
/// Use this from UI click handlers so command resolution stays outside widget code.
pub fn open_image(app_config: &AppConfig, image_path: &str) -> eyre::Result<()> {
    let config = app_config.get()?;
    ExternalOpener::new(config.open_command).open(image_path)
}

/// Starts copying the original image file to the clipboard without blocking the UI thread.
///
/// Use this from UI actions that should place the full-resolution source image on the clipboard.
pub fn copy_image(widget: &impl IsA<gtk::Widget>, image_path: &str) {
    crate::image::clipboard::copy_image(widget, image_path)
}

/// Copies the image path as plain text to the clipboard.
///
/// Use this from UI actions that should expose the filesystem path for the selected image.
pub fn copy_image_path(widget: &impl IsA<gtk::Widget>, image_path: &str) -> eyre::Result<()> {
    crate::image::clipboard::copy_image_path(widget, image_path)
}
