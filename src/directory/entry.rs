//! The responsibility: define directory entries and retain legacy scan entry points.

use crate::{directory::scanner::DirectoryScanner, image::entry::ImageEntry};

/// A directory containing discovered image entries.
///
/// This is the directory-domain model shared by scanning, session state, and UI section creation.
#[derive(Debug, Clone)]
pub struct DirEntry {
    pub dir_path: String,
    pub image_entries: Vec<ImageEntry>,
}

impl DirEntry {
    /// Creates an empty directory entry for the scanner while keeping construction scoped to the crate.
    pub(crate) fn new(dir_path: String) -> Self {
        Self {
            dir_path,
            image_entries: Vec::new(),
        }
    }

    /// Scans a root directory and returns grouped image entries.
    ///
    /// This delegates to `DirectoryScanner` so older call sites can keep using `DirEntry::search`
    /// while scanning logic remains in the scanner module.
    pub fn search(root: &str, max_depth: u32) -> eyre::Result<Vec<DirEntry>> {
        DirectoryScanner::new(max_depth).scan(root)
    }
}
