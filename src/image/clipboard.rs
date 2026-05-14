//! The responsibility: integrate image-related clipboard reads and writes with GTK.

use std::{fs, io::Cursor, path::Path, thread};

use gtk4::{
    self as gtk, gdk, gio,
    glib::{self, prelude::ToValue},
    prelude::*,
};
use image::ImageFormat;

/// Copies an image file path as text to the clipboard associated with the source widget.
pub fn copy_image_path(widget: &impl IsA<gtk::Widget>, image_path: &str) -> eyre::Result<()> {
    let clipboard = widget.clipboard();
    clipboard.set_text(image_path);
    Ok(())
}

/// Loads the original image on a worker thread and publishes clipboard data back on the GTK main context.
pub fn copy_image(widget: &impl IsA<gtk::Widget>, image_path: &str) {
    let clipboard = widget.clipboard();
    let clipboard = glib::SendWeakRef::from(clipboard.downgrade());
    let image_path = image_path.to_owned();
    let main_context = glib::MainContext::default();

    thread::spawn(move || {
        let content = match load_image_clipboard_content(&image_path) {
            Ok(content) => content,
            Err(e) => {
                eprintln!("Failed to load image clipboard content: {e}");
                return;
            }
        };

        main_context.invoke(move || {
            let Some(clipboard) = clipboard.upgrade() else {
                return;
            };

            let provider = build_image_content_provider(content);

            if let Err(e) = clipboard.set_content(Some(&provider)) {
                eprintln!("Failed to set clipboard content: {e}");
                return;
            }
        });
    });
}

/// Clipboard payload prepared off the GTK main thread before being published.
struct PreparedImageClipboardContent {
    original_mime_type: Option<&'static str>,
    original_bytes: Vec<u8>,
    png_bytes: Option<Vec<u8>>,
}

/// Loads clipboard-ready bytes for the original image and a PNG fallback when needed.
fn load_image_clipboard_content(image_path: &str) -> eyre::Result<PreparedImageClipboardContent> {
    let original_bytes = fs::read(image_path)?;
    let original_mime_type = detect_image_mime_type(image_path, &original_bytes);
    let png_bytes = match original_mime_type {
        Some("image/png") => None,
        _ => Some(encode_png_bytes(&original_bytes)?),
    };

    Ok(PreparedImageClipboardContent {
        original_mime_type,
        original_bytes,
        png_bytes,
    })
}

/// Builds clipboard providers for GTK texture consumers plus MIME-based paste targets.
fn build_image_content_provider(content: PreparedImageClipboardContent) -> gdk::ContentProvider {
    let mut providers = Vec::with_capacity(3);

    if let Some(texture_provider) = build_texture_provider(&content) {
        providers.push(texture_provider);
    }

    if let Some(mime_type) = content.original_mime_type {
        providers.push(gdk::ContentProvider::for_bytes(
            mime_type,
            &glib::Bytes::from_owned(content.original_bytes),
        ));
    }

    if let Some(png_bytes) = content.png_bytes {
        providers.push(gdk::ContentProvider::for_bytes(
            "image/png",
            &glib::Bytes::from_owned(png_bytes),
        ));
    }

    if providers.len() == 1 {
        return providers.pop().expect("one provider is available");
    }

    gdk::ContentProvider::new_union(&providers)
}

/// Builds a `GdkTexture` clipboard provider when GTK can decode the source image bytes.
fn build_texture_provider(content: &PreparedImageClipboardContent) -> Option<gdk::ContentProvider> {
    let texture = gdk::Texture::from_bytes(&glib::Bytes::from(content.original_bytes.as_slice()))
        .or_else(|_| {
            content
                .png_bytes
                .as_ref()
                .ok_or_else(|| {
                    glib::Error::new(gio::IOErrorEnum::Failed, "PNG fallback unavailable")
                })
                .and_then(|png_bytes| {
                    gdk::Texture::from_bytes(&glib::Bytes::from(png_bytes.as_slice()))
                })
        })
        .ok()?;

    Some(gdk::ContentProvider::for_value(&texture.to_value()))
}

/// Detects the most appropriate MIME type to offer for the original image bytes.
fn detect_image_mime_type(image_path: &str, image_bytes: &[u8]) -> Option<&'static str> {
    image::guess_format(image_bytes)
        .ok()
        .and_then(image_format_to_mime_type)
        .or_else(|| {
            Path::new(image_path)
                .extension()
                .and_then(|extension| extension.to_str())
                .and_then(extension_to_mime_type)
        })
}

/// Encodes arbitrary source image bytes to PNG for clipboard targets that only accept PNG.
fn encode_png_bytes(image_bytes: &[u8]) -> eyre::Result<Vec<u8>> {
    let image = image::load_from_memory(image_bytes)?;
    let mut cursor = Cursor::new(Vec::new());
    image.write_to(&mut cursor, ImageFormat::Png)?;
    Ok(cursor.into_inner())
}

/// Maps decoded image formats to the MIME type that clipboard consumers expect.
fn image_format_to_mime_type(format: ImageFormat) -> Option<&'static str> {
    match format {
        ImageFormat::Png => Some("image/png"),
        ImageFormat::Jpeg => Some("image/jpeg"),
        ImageFormat::Gif => Some("image/gif"),
        ImageFormat::WebP => Some("image/webp"),
        ImageFormat::Tiff => Some("image/tiff"),
        ImageFormat::Bmp => Some("image/bmp"),
        ImageFormat::Ico => Some("image/x-icon"),
        ImageFormat::Avif => Some("image/avif"),
        _ => None,
    }
}

/// Maps common filename extensions to image MIME types when format guessing is not enough.
fn extension_to_mime_type(extension: &str) -> Option<&'static str> {
    match extension.to_ascii_lowercase().as_str() {
        "png" => Some("image/png"),
        "jpg" | "jpeg" | "jpe" => Some("image/jpeg"),
        "gif" => Some("image/gif"),
        "webp" => Some("image/webp"),
        "tif" | "tiff" => Some("image/tiff"),
        "bmp" => Some("image/bmp"),
        "ico" => Some("image/x-icon"),
        "avif" => Some("image/avif"),
        _ => None,
    }
}
