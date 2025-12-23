use crate::{entry, state::app_config::AppConfig, state::app_state::AppState};
use anyhow::anyhow;
use std::{
    cmp::Ordering,
    path::Path,
    process::{Command, Stdio},
    sync::Arc,
};

pub fn search_and_prepare_entries(
    app_state: Arc<AppState>,
) -> anyhow::Result<(String, Vec<entry::DirEntry>)> {
    let dir_path = app_state.original_dir().map_err(|e| anyhow!(e))?;

    let mut entries = entry::DirEntry::search(&dir_path, app_state.clone())?;
    entries.sort_by(|a, b| a.dir_path.cmp(&b.dir_path));

    app_state
        .set_dir_entries(entries.clone())
        .map_err(|e| anyhow!(e))?;

    let origin_dir = app_state.original_dir().map_err(|e| anyhow!(e))?;
    let dir_entries = app_state.dir_entries().map_err(|e| anyhow!(e))?;

    Ok((origin_dir, dir_entries))
}

pub fn get_relative_path(base_path: &str, path: &str) -> anyhow::Result<String> {
    let base_path = Path::new(base_path).canonicalize()?;
    let path = Path::new(path).canonicalize()?;
    let relative_path = path.strip_prefix(&base_path)?;
    let relative_path = relative_path.to_str().ok_or_else(|| {
        anyhow::anyhow!(
            "Failed to convert path to string: {:?}",
            relative_path.to_str()
        )
    })?;

    if relative_path.is_empty() {
        return Ok(String::from("."));
    }

    Ok(relative_path.to_string())
}

pub fn open_with_xdg_open(image_path: String, app_state: Arc<AppState>) -> anyhow::Result<()> {
    let mut open_command = {
        let app_config = app_state
            .shared
            .config()
            .map_err(|e| anyhow!("Failed to get config: {}", e))?;
        app_config.open_command.clone()
    };
    let index = open_command.iter().position(|x| x == &"<path>".to_string());

    let mut cmd = match index {
        Some(index) => {
            open_command[index] = image_path.clone();
            let first_arg = open_command[0].clone();
            let mut cmd = Command::new(&first_arg);
            cmd.args(&open_command[1..]);
            cmd
        }
        None => {
            let app_config = AppConfig::default();
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

pub fn sort_by_updated_at(a: &str, b: &str, descending: bool) -> anyhow::Result<Ordering> {
    let a_metadata = std::fs::metadata(a)?;
    let b_metadata = std::fs::metadata(b)?;

    let a_modified = a_metadata.modified()?;
    let b_modified = b_metadata.modified()?;

    Ok(match descending {
        true => a_modified.cmp(&b_modified),
        false => b_modified.cmp(&a_modified),
    })
}
