use std::sync::{Arc, RwLock};

use crate::{entry, state::shared::Shared};

pub struct AppState {
    original_dir: RwLock<String>,
    dir_entries: RwLock<Vec<entry::DirEntry>>,
    pub shared: Arc<Shared>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            original_dir: RwLock::new(String::new()),
            dir_entries: RwLock::new(Vec::new()),
            shared: Arc::new(Shared::new()),
        }
    }
}

impl AppState {
    pub fn original_dir(&self) -> Result<String, String> {
        self.original_dir
            .read()
            .map(|dir| dir.clone())
            .map_err(|e| format!("Failed to lock original_dir: {}", e))
    }

    pub fn set_original_dir(&self, dir: String) -> Result<(), String> {
        self.original_dir
            .write()
            .map(|mut d| *d = dir)
            .map_err(|e| format!("Failed to lock original_dir for write: {}", e))
    }

    pub fn dir_entries(&self) -> Result<Vec<entry::DirEntry>, String> {
        self.dir_entries
            .read()
            .map(|entries| entries.clone())
            .map_err(|e| format!("Failed to lock dir_entries: {}", e))
    }

    pub fn set_dir_entries(&self, entries: Vec<entry::DirEntry>) -> Result<(), String> {
        self.dir_entries
            .write()
            .map(|mut e| *e = entries)
            .map_err(|e| format!("Failed to lock dir_entries for write: {}", e))
    }
}
