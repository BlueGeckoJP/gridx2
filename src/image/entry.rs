//! The responsibility: define image file entries discovered from the filesystem.

/// Domain-level image reference.
///
/// This intentionally stores only file identity and no GTK texture so loading and rendering stay
/// outside the directory scan model.
#[derive(Debug, Clone)]
pub struct ImageEntry {
    pub image_path: String,
}
