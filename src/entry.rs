use crate::image_entry::ImageEntry;
use crate::APP_CONFIG;
use anyhow::anyhow;
use std::path;
use std::path::Path;
use walkdir::WalkDir;

#[derive(Debug, Clone)]
pub struct DirEntry {
    pub dir_path: String,
    pub image_entries: Vec<ImageEntry>,
}

impl DirEntry {
    fn new(dir_path: String) -> Self {
        Self {
            dir_path,
            image_entries: Vec::new(),
        }
    }

    pub fn search(root: &str) -> anyhow::Result<Vec<DirEntry>> {
        let max_depth = {
            let app_config = APP_CONFIG
                .lock()
                .map_err(|_| anyhow!("Failed to lock app config"))?;
            app_config.max_depth
        };

        let mut entries: Vec<DirEntry> = Vec::new();
        let max_depth = count_depth(to_absolute(root)?) + max_depth;

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
                .ok_or_else(|| anyhow!("not found parent directory"))?
                .to_string_lossy()
                .to_string();

            let dir_entries_index =
                if let Some(index) = entries.iter().position(|e| e.dir_path == parent) {
                    index
                } else {
                    entries.push(DirEntry::new(parent));
                    entries.len() - 1
                };

            entries[dir_entries_index].image_entries.push(ImageEntry {
                image_path: entry.path().to_string_lossy().to_string(),
                image: None,
            });
        }

        entries.retain(|e| !e.image_entries.is_empty());

        Ok(entries)
    }
}

fn count_depth<T: ToString>(path: T) -> u32 {
    path.to_string()
        .chars()
        .filter(|&c| c == path::MAIN_SEPARATOR)
        .count() as u32
}

fn to_absolute<T: AsRef<Path>>(path: T) -> anyhow::Result<String> {
    Ok(path::absolute(path)?.to_string_lossy().to_string())
}

fn is_image<T: AsRef<Path>>(path: T) -> bool {
    let supported_extensions: [String; 3] = ["png".into(), "jpg".into(), "jpeg".into()];
    let ext = path.as_ref().extension();
    if let Some(ext) = ext {
        supported_extensions.contains(&ext.to_string_lossy().to_string())
    } else {
        false
    }
}
