use serde::Serialize;

#[derive(Serialize)]
pub struct Cpu {
    pub(crate) total: f32,
    pub(crate) cores: Vec<f32>,
    pub(crate) freq_mhz: f32,
    pub(crate) temperature: Option<f32>,
    pub(crate) load: [f64; 3],
}

#[derive(Serialize)]
pub struct Memory {
    pub(crate) total: u64,
    pub(crate) used: u64,
    pub(crate) available: u64,
    pub(crate) cached: u64,
    pub(crate) swap_total: u64,
    pub(crate) swap_used: u64,
}

#[derive(Serialize)]
pub struct DiskRate {
    pub(crate) name: String,
    pub(crate) model: Option<String>,
    pub(crate) read_bps: f64,
    pub(crate) write_bps: f64,
    pub(crate) busy: f64,
}

#[derive(Serialize)]
pub struct Volume {
    pub(crate) mount: String,
    pub(crate) file_system: String,
    pub(crate) total: u64,
    pub(crate) used: u64,
}

#[derive(Serialize)]
pub struct GpuSample {
    pub(crate) id: String,
    pub(crate) vendor: &'static str,
    pub(crate) kind: &'static str,
    pub(crate) name: String,
    pub(crate) util: Option<f64>,
    pub(crate) memory_used: Option<u64>,
    pub(crate) memory_total: Option<u64>,
    pub(crate) temperature: Option<f64>,
    pub(crate) power: Option<f64>,
    pub(crate) power_limit: Option<f64>,
    pub(crate) core_mhz: Option<f64>,
    pub(crate) memory_mhz: Option<f64>,
    pub(crate) fan: Option<f64>,
}

#[derive(Serialize)]
pub struct TempSensor {
    pub(crate) id: String,
    pub(crate) group: String,
    pub(crate) label: String,
    pub(crate) celsius: f64,
    pub(crate) high: Option<f64>,
    pub(crate) critical: Option<f64>,
}

#[derive(Serialize)]
pub struct FanSensor {
    pub(crate) id: String,
    pub(crate) group: String,
    pub(crate) label: String,
    pub(crate) rpm: f64,
}

#[derive(Serialize)]
pub struct Thermal {
    pub(crate) sensors: Vec<TempSensor>,
    pub(crate) fans: Vec<FanSensor>,
}

#[derive(Serialize)]
pub struct Battery {
    pub(crate) name: String,
    pub(crate) percent: f64,
    pub(crate) status: String,
    pub(crate) watts: Option<f64>,
    pub(crate) seconds_left: Option<u64>,
    pub(crate) health: Option<f64>,
    pub(crate) cycles: Option<u64>,
}

#[derive(Serialize)]
pub struct Power {
    pub(crate) cpu_watts: Option<f64>,
    pub(crate) cpu_readable: bool,
    pub(crate) cpu_limit_sustained: Option<f64>,
    pub(crate) cpu_limit_boost: Option<f64>,
    pub(crate) ac_online: Option<bool>,
    pub(crate) batteries: Vec<Battery>,
}

#[derive(Serialize)]
pub struct NetRate {
    pub(crate) name: String,
    pub(crate) kind: &'static str,
    pub(crate) rx_bps: f64,
    pub(crate) tx_bps: f64,
    pub(crate) rx_total: u64,
    pub(crate) tx_total: u64,
    pub(crate) speed_mbps: Option<u32>,
    pub(crate) default_route: bool,
}

#[derive(Serialize)]
pub struct Sample {
    pub(crate) t: u64,
    pub(crate) cpu: Cpu,
    pub(crate) memory: Memory,
    pub(crate) disks: Vec<DiskRate>,
    pub(crate) volumes: Vec<Volume>,
    pub(crate) gpus: Vec<GpuSample>,
    pub(crate) net: Vec<NetRate>,
    pub(crate) thermal: Thermal,
    pub(crate) power: Power,
}
