use tauri::State;

use super::{MonitorService, Sample};
use crate::common::blocking;
use crate::error::Result;

/// One reading of CPU, memory, drives, graphics cards and network.
#[tauri::command]
pub async fn monitor_sample(service: State<'_, MonitorService>) -> Result<Sample> {
    let service = service.inner().clone();
    blocking::run(move || service.sample()).await
}
