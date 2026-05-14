//! The responsibility: sort loaded image presentation data according to app configuration.

use std::cmp::Ordering;

use crate::{
    config::raw_config::{RawConfig, SortOrder},
    image::{metadata::sort_by_updated_at, thumbnail_loader::LoadedImage},
    utils::natural_sort,
};

/// Sorts loaded images without knowing how they were loaded or how they will be rendered.
pub struct ImageSorter;

impl ImageSorter {
    /// Applies the configured ordering to already-loaded thumbnail data.
    pub fn sort(mut image_entries: Vec<LoadedImage>, config: &RawConfig) -> Vec<LoadedImage> {
        match config.sort_order {
            SortOrder::Name => image_entries.sort_by(|a, b| {
                natural_sort(
                    a.image_path.as_str(),
                    b.image_path.as_str(),
                    config.descending,
                )
                .unwrap_or(Ordering::Equal)
            }),
            SortOrder::UpdatedAt => image_entries.sort_by(|a, b| {
                sort_by_updated_at(
                    a.image_path.as_str(),
                    b.image_path.as_str(),
                    config.descending,
                )
                .unwrap_or(Ordering::Equal)
            }),
        }

        image_entries
    }
}
