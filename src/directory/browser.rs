//! The responsibility: coordinate directory selection, scanning, and section view models.

use crate::{
    config::app_config::AppConfig,
    directory::{entry::DirEntry, section::DirectorySection},
    session::Session,
};

/// Owns the directory-browsing use case between UI callbacks and lower-level scanning/session code.
///
/// Use this when the selected directory changes or when the UI needs refreshed section models.
#[derive(Clone)]
pub struct DirectoryBrowser {
    session: Session,
    app_config: AppConfig,
}

impl DirectoryBrowser {
    /// Creates a browser use-case wrapper around shared app state.
    pub fn new(session: Session, app_config: AppConfig) -> Self {
        Self {
            session,
            app_config,
        }
    }

    /// Records the currently selected root directory in the session.
    pub fn select_directory(&self, path: String) -> eyre::Result<()> {
        self.session.set_original_dir(path)
    }

    /// Scans the selected directory, updates session entries, and returns UI-ready sections.
    pub fn load_sections(&self) -> eyre::Result<Vec<DirectorySection>> {
        let max_depth = self.app_config.get()?.max_depth;
        let original_dir = self.session.original_dir()?;

        let mut entries = DirEntry::search(&original_dir, max_depth)?;
        entries.sort_by(|a, b| a.dir_path.cmp(&b.dir_path));

        let sections = DirectorySection::from_entries(&original_dir, &entries)?;
        self.session.replace_dir_entries(entries)?;

        Ok(sections)
    }
}
