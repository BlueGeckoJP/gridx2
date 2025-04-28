use std::path;
use std::path::Path;
use gtk4 as gtk;
use gtk4::{glib, Application, ApplicationWindow};
use gtk4::prelude::{ApplicationExt, ApplicationExtManual, GtkWindowExt};
use walkdir::WalkDir;
use anyhow::{anyhow, Result};

static MAX_DEPTH: u32 = 2;
static BASE_DIR: &str = ".";

fn main() -> glib::ExitCode {
    gtk::init().expect("Failed to initialize GTK");

    let app = Application::builder().application_id("me.bluegecko.gridx2").build();

    let d = DirEntry::search(BASE_DIR.into()).unwrap();
    println!("{d:?}");
    
    app.connect_activate(|app| {
        let window = ApplicationWindow::builder()
            .application(app)
            .default_width(800)
            .default_height(600)
            .title("gridx2")
            .build();
        window.present();
    });
    
    app.run()
}

#[derive(Debug)]
struct DirEntry {
    dir_path: String,
    image_entries: Vec<ImageEntry>,
}

impl DirEntry {
    fn new(dir_path: String) -> Self {
        Self {
            dir_path,
            image_entries: Vec::new(),
        }
    }

    fn search(root: String) -> Result<Vec<DirEntry>> {
        let mut entries: Vec<DirEntry> = Vec::new();
        let max_depth = count_depth(to_absolute(root.clone())?) + MAX_DEPTH;

        let walker = WalkDir::new(root).into_iter();

        let should_process = |entry: &walkdir::DirEntry| -> bool {
            if let Ok(absolute) = to_absolute(entry.path()) {
                if count_depth(absolute) > max_depth {
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

            let dir_entries_index = if let Some(index) = entries.iter().position(|e| e.dir_path == parent) {
                index
            } else {
                entries.push(DirEntry::new(parent));
                entries.len() - 1
            };

            entries[dir_entries_index].image_entries.push(ImageEntry {
                image_path: entry.path().to_string_lossy().to_string(),
                image: gtk::Image::new(),
            });
        }

        entries.retain(|e| !e.image_entries.is_empty());

        Ok(entries)
    }
}

#[derive(Debug)]
struct ImageEntry {
    image_path: String,
    image: gtk::Image,
}

fn count_depth<T: ToString>(path: T) -> u32 {
    path.to_string().chars().filter(|&c| c == path::MAIN_SEPARATOR).count() as u32
}

fn to_absolute<T: AsRef<Path>>(path: T) -> Result<String> {
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