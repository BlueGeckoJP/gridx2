//! The responsibility: provide path helpers used by directory presentation.

use std::path::Path;

/// Returns `path` relative to `base_path` for display in directory section titles.
///
/// Use this when constructing UI-facing labels from absolute filesystem paths.
pub fn get_relative_path(base_path: &str, path: &str) -> eyre::Result<String> {
    let base_path = Path::new(base_path).canonicalize()?;
    let path = Path::new(path).canonicalize()?;
    let relative_path = path.strip_prefix(&base_path)?;
    let relative_path = relative_path
        .to_str()
        .ok_or_else(|| eyre::eyre!("Failed to convert path to string: {:?}", relative_path))?;

    if relative_path.is_empty() {
        return Ok(String::from("."));
    }

    Ok(relative_path.to_string())
}
