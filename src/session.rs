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

    pub fn dir_entries(&self) -> eyre::Result<Vec<DirEntry>> {
        let data = self
            .inner
            .lock()
            .map_err(|e| eyre::eyre!("Failed to lock session data: {e}"))?;
        Ok(data.dir_entries.clone())
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
