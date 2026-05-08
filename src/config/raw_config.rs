use strum::{Display, EnumString};

#[derive(Clone, EnumString, Display)]
#[strum(serialize_all = "snake_case")]
pub enum SortOrder {
    Name,
    UpdatedAt,
}

#[derive(Clone)]
pub struct RawConfig {
    pub max_depth: u32,
    pub thumbnail_size: u32,
    pub open_command: Vec<String>,
    pub dark_mode: bool,
    pub sort_order: SortOrder,
    pub descending: bool,
}

impl Default for RawConfig {
    fn default() -> Self {
        Self {
            max_depth: 2,
            thumbnail_size: 200,
            open_command: vec!["xdg-open".into(), "<path>".into()], // the actual path is assigned to <path>
            dark_mode: true,
            sort_order: SortOrder::Name,
            descending: false,
        }
    }
}
