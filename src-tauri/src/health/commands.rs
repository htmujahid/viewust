use super::model::HealthReport;
use super::report;
use crate::common::blocking;
use crate::error::Result;

#[tauri::command]
pub async fn health_report() -> Result<HealthReport> {
    blocking::run(report::report).await
}
