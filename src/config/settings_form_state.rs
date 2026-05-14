//! The responsibility: convert settings form values into application configuration.

use crate::config::raw_config::{RawConfig, SortOrder};

/// Captures user-editable settings independently of GTK widgets.
///
/// Use this as the boundary between the settings UI and `RawConfig` so parsing/defaulting rules do
/// not live inside widget construction code.
pub struct SettingsFormState {
    max_depth: u32,
    thumbnail_size: u32,
    open_command_text: String,
    sort_order: SortOrder,
    descending: bool,
    dark_mode: bool,
}

impl SettingsFormState {
    /// Builds form state from primitive values read from the settings window controls.
    pub fn new(
        max_depth: u32,
        thumbnail_size: u32,
        open_command_text: String,
        sort_order: SortOrder,
        descending: bool,
        dark_mode: bool,
    ) -> Self {
        Self {
            max_depth,
            thumbnail_size,
            open_command_text,
            sort_order,
            descending,
            dark_mode,
        }
    }

    /// Converts the form state into the persisted runtime configuration.
    pub fn into_raw_config(self) -> RawConfig {
        RawConfig {
            max_depth: self.max_depth,
            thumbnail_size: self.thumbnail_size,
            open_command: self
                .open_command_text
                .split_whitespace()
                .map(|s| s.to_string())
                .collect(),
            sort_order: self.sort_order,
            descending: self.descending,
            dark_mode: self.dark_mode,
        }
    }
}
