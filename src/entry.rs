use crate::{APP_CONFIG, IMAGE_CACHE};
use anyhow::anyhow;
use gtk4::gdk::Texture;
use gtk4::prelude::Cast;
use gtk4::{gdk, glib};
use image::imageops::FilterType;
use image::{GenericImageView, ImageReader};
use std::path;
use std::path::Path;
use walkdir::WalkDir;

#[derive(Debug, Clone)]
pub struct DirEntry {
    pub dir_path: String,
    pub image_entries: Vec<ImageEntry>,
}

#[derive(Debug, Clone)]
pub struct ImageEntry {
    pub image_path: String,
    pub image: Option<Texture>,
}

impl DirEntry {
    fn new(dir_path: String) -> Self {
        Self {
            dir_path,
            image_entries: Vec::new(),
        }
    }

    pub fn search(root: String) -> anyhow::Result<Vec<DirEntry>> {
        let max_depth = {
            let app_config = APP_CONFIG
                .lock()
                .map_err(|_| anyhow!("Failed to lock app config"))?;
            app_config.max_depth
        };

        let mut entries: Vec<DirEntry> = Vec::new();
        let max_depth = count_depth(to_absolute(root.clone())?) + max_depth;

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

    pub fn load_images(&mut self) -> anyhow::Result<()> {
        let thumbnail_size = {
            let app_config = APP_CONFIG
                .lock()
                .map_err(|_| anyhow!("Failed to lock app config"))?;
            app_config.thumbnail_size
        };

        for entry in &mut self.image_entries {
            if entry.image.is_none() {
                let cache_exist = {
                    let image_cache = IMAGE_CACHE
                        .lock()
                        .map_err(|_| anyhow!("Failed to lock image cache"))?;
                    image_cache.get(&entry.image_path).is_some()
                };

                match cache_exist {
                    true => {
                        let image_cache = IMAGE_CACHE
                            .lock()
                            .map_err(|_| anyhow!("Failed to lock image cache"))?;
                        entry.image = image_cache.get(&entry.image_path).cloned();
                    }
                    false => {
                        let img = ImageReader::open(&entry.image_path)?.decode()?;
                        let (width, height) = img.dimensions();
                        let (rw, rh) = calculate_size(width, height, thumbnail_size);
                        let resized = img.resize(rw, rh, FilterType::Lanczos3);
                        let rgba = resized.to_rgba8();
                        let (width, height) = rgba.dimensions();
                        let texture = gdk::MemoryTexture::new(
                            width as i32,
                            height as i32,
                            gdk::MemoryFormat::R8g8b8a8,
                            &glib::Bytes::from(&rgba.into_raw()),
                            (4 * width) as usize,
                        )
                        .upcast::<Texture>();

                        entry.image = Some(texture.clone());

                        let mut image_cache = IMAGE_CACHE
                            .lock()
                            .map_err(|_| anyhow!("Failed to lock image cache"))?;
                        image_cache.insert(&entry.image_path, texture);
                    }
                }
            }
        }

        Ok(())
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

fn calculate_size(mut width: u32, mut height: u32, to: u32) -> (u32, u32) {
    match width > height {
        true => {
            height = (height * to) / width;
            width = to;
            (width, height)
        }
        false => {
            width = (width * to) / height;
            height = to;
            (width, height)
        }
    }
}
