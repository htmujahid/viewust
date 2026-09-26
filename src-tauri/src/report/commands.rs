use super::build;
use crate::common::{blocking, Detail};
use crate::error::Result;

/// The technical report for one device, by the id the map gave it.
#[tauri::command]
pub async fn device_report(id: String) -> Result<Vec<Detail>> {
    blocking::run(move || build(&id)).await
}
