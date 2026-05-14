//! The responsibility: provide file metadata helpers used by image ordering.

use std::{cmp::Ordering, time::SystemTime};

/// Returns the filesystem modification time for a path.
///
/// Use this from sorting code that needs updated-at ordering while keeping filesystem metadata
/// access outside the comparator itself.
pub fn modified_at(path: &str) -> eyre::Result<SystemTime> {
    Ok(std::fs::metadata(path)?.modified()?)
}

/// Compares two already-loaded modification times.
pub fn sort_by_updated_at(a: SystemTime, b: SystemTime, descending: bool) -> Ordering {
    match descending {
        true => a.cmp(&b),
        false => b.cmp(&a),
    }
}
