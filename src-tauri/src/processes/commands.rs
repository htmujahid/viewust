use tauri::State;

use super::{ProcessDetail, ProcessService, Snapshot};
use crate::common::blocking;
use crate::error::Result;

/// Every running process, plus machine-wide totals.
#[tauri::command]
pub async fn process_list(service: State<'_, ProcessService>) -> Result<Snapshot> {
    let service = service.inner().clone();
    blocking::run(move || service.snapshot()).await
}

/// One process in depth.
#[tauri::command]
pub async fn process_detail(service: State<'_, ProcessService>, pid: u32) -> Result<ProcessDetail> {
    let service = service.inner().clone();
    blocking::run(move || service.detail(pid)).await
}
