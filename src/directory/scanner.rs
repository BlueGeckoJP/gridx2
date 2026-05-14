//! The responsibility: scan directories and build image-only directory entries.

use std::path::{self, Path};

use walkdir::WalkDir;

use crate::{directory::entry::DirEntry, image::entry::ImageEntry};

/// Scans a root directory up to a configured depth and groups image paths by parent directory.
///
/// Use this from application/domain code that needs filesystem discovery without taking on UI or
/// session state responsibilities. It intentionally returns plain `DirEntry` data only.
pub struct DirectoryScanner {
    max_depth: u32,
}

impl DirectoryScanner {
    /// Creates a scanner whose depth is relative to the selected root directory.
    pub fn new(max_depth: u32) -> Self {
        Self { max_depth }
    }

    /// Walks the filesystem and returns non-empty image directories.
    ///
    /// Non-readable walk entries are skipped, matching the previous tolerant behavior.
    pub fn scan(&self, root: &str) -> eyre::Result<Vec<DirEntry>> {
        let mut entries: Vec<DirEntry> = Vec::new();
        let max_depth = count_depth(to_absolute(root)?) + self.max_depth;

        let walker = WalkDir::new(root).into_iter();

        let should_process = |entry: &walkdir::DirEntry| -> bool {
            if let Ok(absolute) = to_absolute(entry.path()) {
                if count_depth(absolute) - 1 > max_depth {
                    return false;
                }
                if entry.file_type().is_dir() {
                    return true;
                }
                return is_image(entry.path());
            }
            false
        };

        for entry in walker.filter_entry(should_process) {
            if entry.is_err() {
                continue;
            }
            let entry = entry?;

            if entry.file_type().is_dir() {
                continue;
            }

            let parent = entry
                .path()
                .parent()
                .ok_or_else(|| eyre::eyre!("parent directory not found"))?
                .to_string_lossy();

            let dir_entries_index =
                if let Some(index) = entries.iter().position(|e| e.dir_path.as_str() == parent) {
                    index
                } else {
                    entries.push(DirEntry::new(parent.into_owned()));
                    entries.len() - 1
                };

            entries[dir_entries_index].image_entries.push(ImageEntry {
                image_path: entry.path().to_string_lossy().into_owned(),
            });
        }

        entries.retain(|e| !e.image_entries.is_empty());

        Ok(entries)
    }
}

/// Counts path separators as a lightweight depth metric for walk filtering.
fn count_depth<T: ToString>(path: T) -> u32 {
    path.to_string()
        .chars()
        .filter(|&c| c == path::MAIN_SEPARATOR)
        .count() as u32
}

/// Converts a path to an absolute string for depth comparisons.
fn to_absolute<T: AsRef<Path>>(path: T) -> eyre::Result<String> {
    Ok(path::absolute(path)?.to_string_lossy().into_owned())
}

/// Returns true when the path extension is supported by the image decoder.
fn is_image<T: AsRef<Path>>(path: T) -> bool {
    let supported_extensions: [&str; 20] = [
        "avif", "bmp", "dds", "ff", "gif", "hdr", "ico", "jpg", "jpeg", "jfif", "exr", "png",
        "pbm", "pgm", "ppm", "qoi", "tga", "tif", "tiff", "webp",
    ];
    let ext = path.as_ref().extension();
    if let Some(ext) = ext {
        supported_extensions.contains(&ext.to_string_lossy().as_ref())
    } else {
        false
    }
}
