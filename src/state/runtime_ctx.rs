use std::sync::Arc;

use crate::entry;

// In RuntimeCtx, this effectively becomes something like Arc<RwLock<Arc<T>>>,
// and this implementation is correct.
// The purpose of the second Arc is to avoid performance overhead
// caused by cloning T on every access.
pub struct RuntimeCtx {
    pub original_dir: Arc<String>,
    pub dir_entries: Arc<Vec<entry::DirEntry>>,
}

impl RuntimeCtx {
    pub fn new() -> Self {
        Self {
            original_dir: Arc::new(String::new()),
            dir_entries: Arc::new(Vec::new()),
        }
    }
}
