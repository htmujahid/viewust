use super::{collect, memory, MemoryModules, SystemInfo};
use crate::common::blocking;
use crate::error::Result;

/// Lists the computer's internal parts.
#[tauri::command]
pub async fn system_info() -> Result<SystemInfo> {
    blocking::run(collect).await
}

/// Reads each memory module with `dmidecode`, asking for administrator
/// permission through the desktop's own password prompt. Only runs when the
/// user asks for it. Waiting on that prompt is why this must not block the UI.
#[tauri::command]
pub async fn read_memory_modules() -> Result<MemoryModules> {
    blocking::run(memory::read_modules).await?
}
