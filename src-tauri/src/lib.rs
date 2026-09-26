//! Viewust: a map of everything connected to this computer, and a window into it.
//!
//! Each feature is a module with its own `commands` (the thin Tauri layer, all
//! `async`), its own model types, and the logic behind them. Shared plumbing
//! lives in [`common`].

mod common;
mod connection;
mod error;
mod hardware;
mod monitor;
mod processes;
mod report;
mod system;

pub use error::{AppError, Result};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
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
            monitor::commands::monitor_sample,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
