//! The responsibility: sort loaded image presentation data according to app configuration.

use std::cmp::Ordering;

use crate::{
    config::raw_config::{RawConfig, SortOrder},
    image::{
        metadata::{modified_at, sort_by_updated_at},
        thumbnail_loader::LoadedImage,
    },
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
            SortOrder::UpdatedAt => {
                // Cache modification times once so sorting does not repeatedly hit the filesystem.
                let mut keyed_entries: Vec<_> = image_entries
                    .into_iter()
                    .map(|image_entry| {
                        let modified = modified_at(image_entry.image_path.as_str()).ok();
                        (image_entry, modified)
                    })
                    .collect();

                keyed_entries.sort_by(|(a, a_modified), (b, b_modified)| {
                    match (a_modified, b_modified) {
                        (Some(a_modified), Some(b_modified)) => {
                            sort_by_updated_at(*a_modified, *b_modified, config.descending)
                        }
                        _ => natural_sort(
                            a.image_path.as_str(),
                            b.image_path.as_str(),
                            config.descending,
                        )
                        .unwrap_or(Ordering::Equal),
                    }
                });

                image_entries = keyed_entries
                    .into_iter()
                    .map(|(image_entry, _)| image_entry)
                    .collect();
            }
        }

        image_entries
    }
}
