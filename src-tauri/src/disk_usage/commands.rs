use tauri::State;

use super::model::{Devices, DirectoryUsage};
use super::{devices, directory, mount, open, UsageService};
use crate::common::blocking;
use crate::error::Result;

#[tauri::command]
pub async fn disk_devices() -> Result<Devices> {
    blocking::run(devices::snapshot).await?
}

#[tauri::command]
pub async fn directory_usage(
    service: State<'_, UsageService>,
    path: String,
    refresh: bool,
) -> Result<DirectoryUsage> {
    let service = service.inner().clone();
    blocking::run(move || directory::directory(&service, &path, refresh)).await?
}

#[tauri::command]
pub async fn path_open(path: String) -> Result<()> {
    blocking::run(move || open::open(&path)).await?
}

#[tauri::command]
pub async fn disk_mount(device: String) -> Result<String> {
    blocking::run(move || mount::mount(&device)).await?
}
