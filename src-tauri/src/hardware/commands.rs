use tauri::AppHandle;

use super::model::HardwareInfo;
use super::{computer_details, displays, peripherals};
use crate::common::blocking;
use crate::connection;
use crate::error::Result;

/// Scans for everything connected from outside.
#[tauri::command]
pub async fn hardware_info(app: AppHandle) -> Result<HardwareInfo> {
    // Monitor handles come from the windowing system; the rest is slow I/O.
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
