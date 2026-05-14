//! The responsibility: convert directory entries into UI-facing section data.

use crate::directory::{entry::DirEntry, path::get_relative_path};

/// View model for one collapsible directory section in the main image list.
///
/// It stores preformatted title data and counts so GTK rendering does not need filesystem context.
pub struct DirectorySection {
    title: String,
    image_count: usize,
    directory_path: String,
}

impl DirectorySection {
    /// Builds display sections from scanned directory entries relative to the selected root.
    pub fn from_entries(
        original_dir: &str,
        entries: &[DirEntry],
    ) -> eyre::Result<Vec<DirectorySection>> {
        entries
            .iter()
            .map(|entry| {
                Ok(DirectorySection {
                    title: format!(
                        "{} | {} entries",
                        get_relative_path(original_dir, &entry.dir_path)?,
                        entry.image_entries.len()
                    ),
                    image_count: entry.image_entries.len(),
                    directory_path: entry.dir_path.clone(),
                })
            })
            .collect()
    }

    /// Returns the title shown on the accordion header.
    pub fn title(&self) -> &str {
        &self.title
    }

    /// Returns the number of images represented by this section.
    pub fn image_count(&self) -> usize {
        self.image_count
    }

    /// Returns the absolute directory path used to resolve the backing `DirEntry`.
    pub fn directory_path(&self) -> String {
        self.directory_path.clone()
    }
}
