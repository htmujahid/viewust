use serde::Serialize;

use crate::common::Detail;

#[derive(Serialize, Clone)]
pub struct ServiceRow {
    pub(crate) unit: String,
    pub(crate) name: String,
    pub(crate) description: String,
    pub(crate) load: String,
    pub(crate) active: String,
    pub(crate) sub: String,
    pub(crate) enabled: String,
    pub(crate) main_pid: Option<u32>,
    pub(crate) memory: Option<u64>,
}

#[derive(Serialize, Default)]
pub struct Overview {
    pub(crate) total: usize,
    pub(crate) running: usize,
    pub(crate) exited: usize,
    pub(crate) failed: usize,
    pub(crate) inactive: usize,
    pub(crate) enabled: usize,
    pub(crate) memory: u64,
}

#[derive(Serialize)]
pub struct Snapshot {
    pub(crate) available: bool,
    pub(crate) overview: Overview,
    pub(crate) services: Vec<ServiceRow>,
}

#[derive(Serialize)]
pub struct ServiceDetail {
    pub(crate) unit: String,
    pub(crate) found: bool,
    pub(crate) details: Vec<Detail>,
    pub(crate) logs: Vec<String>,
    pub(crate) logs_note: Option<String>,
}
