mod accounts;
mod common;
mod connection;
mod error;
mod filesystems;
mod hardware;
mod monitor;
mod processes;
mod report;
mod services;
mod system;

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
        .invoke_handler(tauri::generate_handler![
            hardware::commands::hardware_info,
            report::commands::device_report,
            system::commands::system_info,
            system::commands::read_memory_modules,
            processes::commands::process_list,
            processes::commands::process_detail,
            processes::commands::process_signal,
            processes::commands::namespace_list,
            accounts::commands::account_list,
            filesystems::commands::filesystem_list,
            filesystems::commands::filesystem_action,
            services::commands::service_list,
            services::commands::service_detail,
            services::commands::service_action,
            monitor::commands::monitor_sample,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
