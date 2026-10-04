use super::{list, Snapshot};
use crate::common::blocking;
use crate::error::Result;

#[tauri::command]
pub async fn filesystem_list() -> Result<Snapshot> {
    blocking::run(list::snapshot).await?
}
