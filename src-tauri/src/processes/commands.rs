use tauri::State;

use super::{namespaces, signal, Namespaces, ProcessDetail, ProcessService, Snapshot};
use crate::common::blocking;
use crate::error::Result;

#[tauri::command]
pub async fn process_list(service: State<'_, ProcessService>) -> Result<Snapshot> {
    let service = service.inner().clone();
    blocking::run(move || service.snapshot()).await
}

#[tauri::command]
pub async fn process_detail(service: State<'_, ProcessService>, pid: u32) -> Result<ProcessDetail> {
    let service = service.inner().clone();
    blocking::run(move || service.detail(pid)).await
}

#[tauri::command]
pub async fn process_signal(pid: u32, signal: String) -> Result<()> {
    blocking::run(move || signal::send(pid, &signal)).await?
}

#[tauri::command]
pub async fn namespace_list() -> Result<Namespaces> {
    blocking::run(namespaces::snapshot).await
}
