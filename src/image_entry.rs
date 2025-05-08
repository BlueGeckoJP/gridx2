use crate::{APP_CONFIG, IMAGE_CACHE};
use anyhow::anyhow;
use gtk4::gdk::Texture;
use gtk4::prelude::Cast;
use gtk4::{gdk, glib};
use image::imageops::FilterType;
use image::{GenericImageView, ImageReader};

#[derive(Debug, Clone)]
pub struct ImageEntry {
    pub image_path: String,
    pub image: Option<Texture>,
}

impl ImageEntry {
    pub fn load_image(&mut self) -> anyhow::Result<()> {
        if self.image.is_some() {
            return Ok(());
        }

        let thumbnail_size = {
            let app_config = APP_CONFIG
                .lock()
                .map_err(|_| anyhow!("Failed to lock app config"))?;
            app_config.thumbnail_size
        };

        let cache_hit = {
            let image_cache = match IMAGE_CACHE.lock() {
                Ok(cache) => cache,
                Err(_) => return Err(anyhow!("Failed to lock image cache")),
            };
            image_cache.get(&self.image_path).cloned()
        };

        if let Some(texture) = cache_hit {
            self.image = Some(texture.clone());
            return Ok(());
        }

        if let Ok(texture) = self.load_and_resize_image(thumbnail_size) {
            self.image = Some(texture.clone());

            if let Ok(mut image_cache) = IMAGE_CACHE.lock() {
                image_cache.insert(&self.image_path, texture);
            }
        }

        Ok(())
    }

    fn load_and_resize_image(&self, thumbnail_size: u32) -> anyhow::Result<Texture> {
        let path = &self.image_path;
        let img = ImageReader::open(path)?.decode()?;
        let (width, height) = img.dimensions();
        let (rw, rh) = self.calculate_size(width, height, thumbnail_size);
        let resized = img.resize(rw, rh, FilterType::Triangle);
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

        Ok(texture)
    }

    fn calculate_size(&self, mut width: u32, mut height: u32, to: u32) -> (u32, u32) {
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
}
