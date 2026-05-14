//! The responsibility: provide file metadata helpers used by image ordering.

use std::cmp::Ordering;

/// Compares two image paths by filesystem modification time.
///
/// Use this from sorting code that needs updated-at ordering while keeping filesystem metadata
/// access outside the sorter itself.
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
