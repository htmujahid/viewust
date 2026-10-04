use super::model::{EnvVar, KernelModule, ModuleInfo, OsSummary, Packages};
use super::{environment, info, modules, packages};
use crate::common::blocking;
use crate::error::Result;

#[tauri::command]
pub async fn os_summary() -> Result<OsSummary> {
    blocking::run(info::summary).await
}

#[tauri::command]
pub async fn kernel_modules() -> Result<Vec<KernelModule>> {
    blocking::run(modules::list).await
}

#[tauri::command]
pub async fn kernel_module_info(name: String) -> Result<ModuleInfo> {
    blocking::run(move || modules::info(&name)).await
}

#[tauri::command]
pub async fn os_packages() -> Result<Packages> {
    blocking::run(packages::list).await
}

#[tauri::command]
pub async fn os_environment() -> Result<Vec<EnvVar>> {
    blocking::run(environment::list).await
}
