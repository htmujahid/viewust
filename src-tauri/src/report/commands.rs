use super::build;
use crate::common::{blocking, Detail};
use crate::error::Result;

#[tauri::command]
pub async fn device_report(id: String) -> Result<Vec<Detail>> {
    blocking::run(move || build(&id)).await
}
