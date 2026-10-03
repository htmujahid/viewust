mod accounts;
mod common;
mod connection;
mod error;
mod hardware;
mod monitor;
mod processes;
mod report;
mod services;
mod system;

pub use error::{AppError, Result};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    #[cfg(target_os = "linux")]
    if std::env::var_os("WEBKIT_DISABLE_DMABUF_RENDERER").is_none() {
        std::env::set_var("WEBKIT_DISABLE_DMABUF_RENDERER", "1");
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
            processes::commands::namespace_list,
            accounts::commands::account_list,
            services::commands::service_list,
            services::commands::service_detail,
            monitor::commands::monitor_sample,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
