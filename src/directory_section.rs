use crate::{
    config::app_config::AppConfig, entry::DirEntry, file_utils::get_relative_path, session::Session,
};

pub struct DirectorySection {
    title: String,
    image_count: usize,
    index: usize,
}

impl DirectorySection {
    pub fn load_sections(
        session: Session,
        app_config: AppConfig,
    ) -> eyre::Result<Vec<DirectorySection>> {
        let max_depth = app_config.get()?.max_depth;
        let original_dir = session.original_dir()?;

        let mut entries = DirEntry::search(&original_dir, max_depth)?;
        entries.sort_by(|a, b| a.dir_path.cmp(&b.dir_path));

        session.set_original_dir(original_dir.clone())?;
        session.replace_dir_entries(entries.clone())?;

        entries
            .iter()
            .enumerate()
            .map(|(index, entry)| {
                Ok(DirectorySection {
                    title: format!(
                        "{} | {} entries",
                        get_relative_path(&original_dir, &entry.dir_path)?,
                        entry.image_entries.len()
                    ),
                    image_count: entry.image_entries.len(),
                    index,
                })
            })
            .collect()
    }

    pub fn title(&self) -> &str {
        &self.title
    }

    pub fn image_count(&self) -> usize {
        self.image_count
    }

    pub fn index(&self) -> usize {
        self.index
    }
}
