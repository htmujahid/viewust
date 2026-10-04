use super::model::{
    Cgroups, Connections, Containers, EnvVar, KernelModule, ModuleInfo, OsLogins, OsLogs, OsMemory,
    OsNetwork, OsSecurity, OsSummary, Packages, VirtOverview, Vms,
};
use super::{
    cgroups, containers, environment, info, logins, logs, memory, modules, network, packages,
    security, virt, vms,
};
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

#[tauri::command]
pub async fn os_memory() -> Result<OsMemory> {
    blocking::run(memory::snapshot).await
}

#[tauri::command]
pub async fn os_security() -> Result<OsSecurity> {
    blocking::run(security::snapshot).await
}

#[tauri::command]
pub async fn os_cgroups() -> Result<Cgroups> {
    blocking::run(cgroups::snapshot).await
}

#[tauri::command]
pub async fn os_network() -> Result<OsNetwork> {
    blocking::run(network::snapshot).await
}

#[tauri::command]
pub async fn os_connections() -> Result<Connections> {
    blocking::run(network::connections).await
}

#[tauri::command]
pub async fn os_logs() -> Result<OsLogs> {
    blocking::run(logs::snapshot).await
}

#[tauri::command]
pub async fn os_logins() -> Result<OsLogins> {
    blocking::run(logins::snapshot).await
}

#[tauri::command]
pub async fn os_containers() -> Result<Containers> {
    blocking::run(containers::snapshot).await
}

#[tauri::command]
pub async fn os_vms() -> Result<Vms> {
    blocking::run(vms::snapshot).await
}

#[tauri::command]
pub async fn virt_overview() -> Result<VirtOverview> {
    blocking::run(virt::overview).await
}
