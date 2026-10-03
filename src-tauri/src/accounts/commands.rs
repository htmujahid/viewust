use super::{users, Accounts};
use crate::common::blocking;
use crate::error::Result;

#[tauri::command]
pub async fn account_list() -> Result<Accounts> {
    blocking::run(users::snapshot).await
}
