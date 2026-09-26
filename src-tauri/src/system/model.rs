//! What the internals page shows.

use serde::Serialize;

use crate::common::Detail;

#[derive(Serialize)]
pub struct Component {
    pub(crate) id: String,
    pub(crate) kind: &'static str,
    pub(crate) name: String,
    pub(crate) subtitle: Option<String>,
    pub(crate) details: Vec<Detail>,
}

#[derive(Serialize)]
pub struct SystemInfo {
    pub(crate) computer_name: String,
    pub(crate) components: Vec<Component>,
}

#[derive(Serialize)]
pub struct MemoryModules {
    pub(crate) modules: Vec<Component>,
    /// Total memory slots on the board, including empty ones.
    pub(crate) slots: u32,
}
