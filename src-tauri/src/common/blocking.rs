//! Keeping slow reads off the UI thread.
//!
//! Tauri runs plain `fn` commands on the main thread, so a command that spawns
//! `nvidia-smi` or waits on a password prompt would freeze the window. Commands
//! are `async` and hand their work to this helper instead.

use crate::error::{AppError, Result};

/// Runs blocking `work` on a worker thread and awaits it.
pub async fn run<T, F>(work: F) -> Result<T>
where
    T: Send + 'static,
    F: FnOnce() -> T + Send + 'static,
{
    tauri::async_runtime::spawn_blocking(work)
        .await
        .map_err(|e| AppError::Interrupted(e.to_string()))
}
