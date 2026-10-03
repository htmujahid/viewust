use tauri::AppHandle;

use super::model::HardwareInfo;
use super::{computer_details, displays, peripherals};
use crate::common::blocking;
use crate::connection;
use crate::error::Result;

#[tauri::command]
pub async fn hardware_info(app: AppHandle) -> Result<HardwareInfo> {
    let monitors = app.available_monitors().unwrap_or_default();
    let primary = app.primary_monitor().ok().flatten();

    blocking::run(move || HardwareInfo {
        computer_name: sysinfo::System::host_name().unwrap_or_else(|| "This computer".into()),
        computer_details: computer_details(),
        connection: connection::connection(),
        peripherals: peripherals(),
        displays: displays::list(monitors, primary),
    })
    .await
}
