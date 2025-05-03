pub struct AppConfig {
    pub max_depth: u32,
    pub thumbnail_size: u32,
    pub open_command: Vec<String>,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            max_depth: 2,
            thumbnail_size: 200,
            open_command: vec!["xdg-open".into(), "<path>".into()], // the actual path is assigned to <path>
        }
    }
}
