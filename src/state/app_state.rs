use std::sync::{Arc, RwLock};

use crate::state::{runtime_ctx::RuntimeCtx, shared::Shared};

pub struct AppState {
    runtime_ctx: RwLock<RuntimeCtx>,
    pub shared: Arc<Shared>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            runtime_ctx: RwLock::new(RuntimeCtx::new()),
            shared: Arc::new(Shared::new()),
        }
    }
}

impl AppState {
    pub fn with_runtime_ctx<R>(&self, f: impl FnOnce(&RuntimeCtx) -> R) -> eyre::Result<R> {
        let guard = self
            .runtime_ctx
            .read()
            .map_err(|e| eyre::eyre!("Failed to lock runtime_ctx: {}", e))?;
        Ok(f(&guard))
    }

    pub fn update_runtime_ctx<R>(&self, f: impl FnOnce(&mut RuntimeCtx) -> R) -> eyre::Result<R> {
        let mut guard = self
            .runtime_ctx
            .write()
            .map_err(|e| eyre::eyre!("Failed to lock runtime_ctx for write: {}", e))?;
        Ok(f(&mut guard))
    }
}
