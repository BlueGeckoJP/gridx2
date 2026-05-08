use crate::{config::raw_config::RawConfig, entry, session::Session};
use std::{
    cmp::Ordering,
    path::Path,
    process::{Command, Stdio},
};

pub fn search_and_prepare_entries(session: Session, max_depth: u32) -> eyre::Result<()> {
    let dir_path = session.original_dir()?;

    let mut entries = entry::DirEntry::search(&dir_path, max_depth)?;
    entries.sort_by(|a, b| a.dir_path.cmp(&b.dir_path));

    session.set_original_dir(dir_path)?;
    session.replace_dir_entries(entries)?;

    Ok(())
}

pub fn get_relative_path(base_path: &str, path: &str) -> eyre::Result<String> {
    let base_path = Path::new(base_path).canonicalize()?;
    let path = Path::new(path).canonicalize()?;
    let relative_path = path.strip_prefix(&base_path)?;
    let relative_path = relative_path.to_str().ok_or_else(|| {
        eyre::eyre!(
            "Failed to convert path to string: {:?}",
            relative_path.to_str()
        )
    })?;

    if relative_path.is_empty() {
        return Ok(String::from("."));
    }

    Ok(relative_path.to_string())
}

pub fn open_with_xdg_open(image_path: String, mut open_command: Vec<String>) -> eyre::Result<()> {
    let index = open_command.iter().position(|x| x == "<path>");

    let mut cmd = match index {
        Some(index) => {
            open_command[index] = image_path.clone();
            let first_arg = open_command[0].clone();
            let mut cmd = Command::new(&first_arg);
            cmd.args(&open_command[1..]);
            cmd
        }
        None => {
            let app_config = RawConfig::default();
            let first_arg = app_config.open_command[0].clone();
            let mut cmd = Command::new(&first_arg);
            cmd.args(&app_config.open_command[1..]);
            cmd
        }
    };

    cmd.stdout(Stdio::null())
        .stderr(Stdio::null())
        .stdin(Stdio::null());

    cmd.spawn()?;

    Ok(())
}

pub fn sort_by_updated_at(a: &str, b: &str, descending: bool) -> eyre::Result<Ordering> {
    let a_metadata = std::fs::metadata(a)?;
    let b_metadata = std::fs::metadata(b)?;

    let a_modified = a_metadata.modified()?;
    let b_modified = b_metadata.modified()?;

    Ok(match descending {
        true => a_modified.cmp(&b_modified),
        false => b_modified.cmp(&a_modified),
    })
}
