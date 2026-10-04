use super::{actions, list, Snapshot};
use crate::common::blocking;
use crate::error::{AppError, Result};

#[tauri::command]
pub async fn filesystem_list() -> Result<Snapshot> {
    blocking::run(list::snapshot).await?
}

#[tauri::command]
pub async fn filesystem_action(mount: String, action: String) -> Result<()> {
    blocking::run(move || match action.as_str() {
        "open" => actions::open(&mount),
        "unmount" => actions::unmount(&mount),
        other => Err(AppError::Other(format!("Unknown action “{other}”"))),
    })
    .await?
}
