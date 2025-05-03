pub struct AppConfig {
    pub max_depth: u32,
    pub thumbnail_size: u32,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            max_depth: 2,
            thumbnail_size: 200,
        }
    }
}
