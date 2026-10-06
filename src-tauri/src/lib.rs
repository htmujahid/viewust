mod accounts;
mod common;
mod connection;
mod disk_usage;
mod error;
mod hardware;
mod health;
mod monitor;
mod os;
mod processes;
mod report;
mod services;
mod system;
mod tray;

pub use error::{AppError, Result};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // WebKitGTK's GPU compositing can draw page layers at the wrong offsets, which scatters the
    // layout across the window. Falling back to its simpler paint path avoids it.
    for var in [
        "WEBKIT_DISABLE_DMABUF_RENDERER",
        "WEBKIT_DISABLE_COMPOSITING_MODE",
    ] {
        if std::env::var_os(var).is_none() {
            std::env::set_var(var, "1");
        }
    }
    tauri::Builder::default()
        .manage(processes::ProcessService::default())
        .manage(monitor::MonitorService::default())
        .manage(disk_usage::UsageService::default())
        .setup(|app| {
            tray::setup(app)?;
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            hardware::commands::hardware_info,
            health::commands::health_report,
            report::commands::device_report,
            system::commands::system_info,
            system::commands::read_memory_modules,
            processes::commands::process_list,
            processes::commands::process_detail,
            processes::commands::process_signal,
            processes::commands::namespace_list,
            accounts::commands::account_list,
            disk_usage::commands::disk_devices,
            disk_usage::commands::directory_usage,
            disk_usage::commands::path_open,
            disk_usage::commands::disk_mount,
            disk_usage::commands::filesystem_list,
            services::commands::service_list,
            services::commands::service_detail,
            services::commands::service_action,
            monitor::commands::monitor_sample,
            monitor::commands::audio_sample,
            monitor::commands::speed_test,
            os::commands::os_summary,
            os::commands::kernel_modules,
            os::commands::kernel_module_info,
            os::commands::os_packages,
            os::commands::os_environment,
            os::commands::os_memory,
            os::commands::os_security,
            os::commands::os_cgroups,
            os::commands::os_network,
            os::commands::os_connections,
            os::commands::os_logs,
            os::commands::os_logins,
            os::commands::os_containers,
            os::commands::os_vms,
            os::commands::virt_overview,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
