use std::sync::{Arc, Mutex};

use crate::directory::entry::DirEntry;

#[derive(Default)]
struct SessionData {
    original_dir: String,
    dir_entries: Vec<DirEntry>,
}

#[derive(Clone)]
pub struct Session {
    inner: Arc<Mutex<SessionData>>,
}

impl Session {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(SessionData::default())),
        }
    }

    pub fn set_original_dir(&self, dir: String) -> eyre::Result<()> {
        let mut data = self
            .inner
            .lock()
            .map_err(|e| eyre::eyre!("Failed to lock session data: {e}"))?;
        data.original_dir = dir;
        Ok(())
    }

    pub fn original_dir(&self) -> eyre::Result<String> {
        let data = self
            .inner
            .lock()
            .map_err(|e| eyre::eyre!("Failed to lock session data: {e}"))?;
        Ok(data.original_dir.clone())
    }

    /// Returns a clone of the directory entries. This is not memory efficient, so consider using find_dir_entry or replace_dir_entries instead.
    #[deprecated(note = "Use find_dir_entry instead for better performance and memory efficiency")]
    #[allow(unused)]
    pub fn dir_entries(&self) -> eyre::Result<Vec<DirEntry>> {
        let data = self
            .inner
            .lock()
            .map_err(|e| eyre::eyre!("Failed to lock session data: {e}"))?;
        Ok(data.dir_entries.clone())
    }

    /// Finds a directory entry by its path and returns a clone wrapped in Arc.
    /// Arc wraps DirEntry to reduce clone costs. However, this function still needs to clone the matching DirEntry for dir_path, 
    /// so further optimization would require changing how SessionData::dir_entries stores its data.
    pub fn find_dir_entry(&self, dir_path: &str) -> eyre::Result<Option<Arc<DirEntry>>> {
        let data = self
            .inner
            .lock()
            .map_err(|e| eyre::eyre!("Failed to lock session data: {e}"))?;

        let entry = data
            .dir_entries
            .iter()
            .find(|entry| entry.dir_path == dir_path)
            .map(|entry| Arc::new(entry.clone()));

        Ok(entry)
    }

    pub fn replace_dir_entries(&self, new_entries: Vec<DirEntry>) -> eyre::Result<()> {
        let mut data = self
            .inner
            .lock()
            .map_err(|e| eyre::eyre!("Failed to lock session data: {e}"))?;
        data.dir_entries = new_entries;
        Ok(())
    }
}
