use gtk4::gdk::Texture;
use std::collections::HashMap;

pub struct ImageCache {
    map: HashMap<String, Texture>,
    limit: usize,
}

impl ImageCache {
    pub fn new(limit: usize) -> Self {
        Self {
            map: HashMap::new(),
            limit,
        }
    }

    pub fn insert<S: ToString>(&mut self, path: S, value: Texture) {
        if self.map.len() >= self.limit {
            if let Some(first_key) = self.map.keys().next().cloned() {
                self.map.remove(&first_key);
            }
        }
        self.map.insert(path.to_string(), value);
    }

    pub fn get<S: ToString>(&self, path: S) -> Option<&Texture> {
        self.map.get(&path.to_string())
    }

    pub fn _remove<S: ToString>(&mut self, path: S) -> Option<Texture> {
        self.map.remove(&path.to_string())
    }
}
