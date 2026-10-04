use tauri::State;

use super::{MonitorService, Sample};
use crate::common::blocking;
use crate::error::Result;

#[tauri::command]
pub async fn monitor_sample(service: State<'_, MonitorService>) -> Result<Sample> {
    let service = service.inner().clone();
    blocking::run(move || service.sample()).await
}

#[tauri::command]
pub async fn audio_sample() -> Result<crate::monitor::audio::AudioSample> {
    blocking::run(crate::monitor::audio::sample).await
}

#[tauri::command]
pub async fn speed_test() -> Result<crate::monitor::speedtest::SpeedTest> {
    blocking::run(crate::monitor::speedtest::run_test).await
}
