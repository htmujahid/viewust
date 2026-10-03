use serde::Serialize;

use crate::common::Detail;

#[derive(Serialize)]
pub struct ProcessRow {
    pub(crate) pid: u32,
    pub(crate) parent: Option<u32>,
    pub(crate) name: String,
    pub(crate) user: String,
    pub(crate) cpu: f32,
    pub(crate) memory: u64,
    pub(crate) virtual_memory: u64,
    pub(crate) threads: u32,
    pub(crate) state: &'static str,
    pub(crate) kernel: bool,
    pub(crate) run_time: u64,
}

#[derive(Serialize)]
pub struct Overview {
    pub(crate) processes: usize,
    pub(crate) running: usize,
    pub(crate) threads: u32,
    pub(crate) cpu: f32,
    pub(crate) cpu_count: usize,
    pub(crate) load: [f64; 3],
    pub(crate) memory_total: u64,
    pub(crate) memory_used: u64,
    pub(crate) memory_available: u64,
    pub(crate) swap_total: u64,
    pub(crate) swap_used: u64,
    pub(crate) uptime: u64,
}

#[derive(Serialize)]
pub struct Snapshot {
    pub(crate) overview: Overview,
    pub(crate) processes: Vec<ProcessRow>,
}

#[derive(Serialize, Default)]
pub struct MemoryBreakdown {
    pub(crate) requested: u64,
    pub(crate) peak_requested: u64,
    pub(crate) resident: u64,
    pub(crate) peak_resident: u64,
    pub(crate) anonymous: u64,
    pub(crate) file_backed: u64,
    pub(crate) shared: u64,
    pub(crate) swapped: u64,
    pub(crate) data: u64,
    pub(crate) stack: u64,
    pub(crate) code: u64,
    pub(crate) libraries: u64,
    pub(crate) page_tables: u64,
    pub(crate) locked: u64,
    pub(crate) proportional: Option<u64>,
    pub(crate) private: Option<u64>,
    pub(crate) shared_pages: Option<u64>,
}

#[derive(Serialize)]
pub struct Region {
    pub(crate) name: String,
    pub(crate) size: u64,
    pub(crate) resident: u64,
}

#[derive(Serialize)]
pub struct Child {
    pub(crate) pid: u32,
    pub(crate) name: String,
    pub(crate) memory: u64,
}

#[derive(Serialize)]
pub struct ProcessDetail {
    pub(crate) pid: u32,
    pub(crate) running: bool,
    pub(crate) restricted: bool,
    pub(crate) memory: Option<MemoryBreakdown>,
    pub(crate) regions: Vec<Region>,
    pub(crate) children: Vec<Child>,
    pub(crate) details: Vec<Detail>,
}

#[derive(Serialize, Clone)]
pub struct NsProcess {
    pub(crate) pid: u32,
    pub(crate) name: String,
    pub(crate) user: String,
}

#[derive(Serialize, Clone)]
pub struct NamespaceRow {
    pub(crate) kind: String,
    pub(crate) id: u64,
    pub(crate) processes: u32,
    pub(crate) sample: Vec<NsProcess>,
    pub(crate) current: bool,
}

#[derive(Serialize)]
pub struct Namespaces {
    pub(crate) supported: bool,
    pub(crate) note: Option<String>,
    pub(crate) inspected: usize,
    pub(crate) total: usize,
    pub(crate) namespaces: Vec<NamespaceRow>,
}
