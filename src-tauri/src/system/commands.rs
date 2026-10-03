use super::{collect, memory, MemoryModules, SystemInfo};
use crate::common::blocking;
use crate::error::Result;

#[tauri::command]
pub async fn system_info() -> Result<SystemInfo> {
    blocking::run(collect).await
}

#[tauri::command]
pub async fn read_memory_modules() -> Result<MemoryModules> {
    blocking::run(memory::read_modules).await?
}
