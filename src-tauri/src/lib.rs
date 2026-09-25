mod connection;
mod detail;
mod hardware;
mod monitor;
mod processes;
mod report;
mod system;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            hardware::hardware_info,
            report::device_report,
            system::system_info,
            system::read_memory_modules,
            processes::process_list,
            processes::process_detail,
            monitor::monitor_sample
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
