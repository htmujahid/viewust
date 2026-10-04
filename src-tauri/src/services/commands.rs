use super::{control, detail, list, ServiceDetail, Snapshot};
use crate::common::blocking;
use crate::error::Result;

#[tauri::command]
pub async fn service_list() -> Result<Snapshot> {
    blocking::run(list::snapshot).await
}

#[tauri::command]
pub async fn service_detail(unit: String) -> Result<ServiceDetail> {
    blocking::run(move || detail::detail(&unit)).await
}

#[tauri::command]
pub async fn service_action(unit: String, action: String) -> Result<()> {
    blocking::run(move || control::run(&unit, &action)).await?
}
