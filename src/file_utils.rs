use crate::{APP_CONFIG, AppState, app_config::AppConfig, entry};
use anyhow::anyhow;
use std::{
    cmp::Ordering,
    path::Path,
    process::{Command, Stdio},
    sync::{Arc, Mutex},
};

pub fn search_and_prepare_entries(
    app_state: Arc<Mutex<AppState>>,
) -> anyhow::Result<(String, Vec<entry::DirEntry>)> {
    let dir_path = {
        let guard = app_state.lock().map_err(|_| anyhow!("Failed to lock"))?;
        guard.original_dir.clone()
    };

    let mut entries = entry::DirEntry::search(&dir_path)?;
    entries.sort_by(|a, b| a.dir_path.cmp(&b.dir_path));

    let (origin_dir, dir_entries) = {
        let mut guard = app_state.lock().map_err(|_| anyhow!("Failed to lock"))?;
        guard.dir_entries = entries;

        (guard.original_dir.clone(), guard.dir_entries.clone())
    };

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

pub fn open_with_xdg_open(image_path: String) -> anyhow::Result<()> {
    let mut open_command = {
        let app_config = APP_CONFIG
            .read()
            .map_err(|_| anyhow!("Failed to lock app config"))?;
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
