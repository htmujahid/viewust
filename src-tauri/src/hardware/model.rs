use serde::Serialize;

use crate::common::Detail;

#[derive(Serialize)]
pub struct Peripheral {
    pub(crate) id: String,
    pub(crate) via: Option<String>,
    pub(crate) wireless: bool,
    pub(crate) name: String,
    pub(crate) manufacturer: Option<String>,
    pub(crate) kind: &'static str,
    pub(crate) connection: String,
    pub(crate) vendor_id: String,
    pub(crate) product_id: String,
    pub(crate) serial_number: Option<String>,
    pub(crate) details: Vec<Detail>,
}

#[derive(Serialize)]
pub struct Display {
    pub(crate) name: String,
    pub(crate) connector: Option<String>,
    pub(crate) width_cm: Option<u32>,
    pub(crate) width: u32,
    pub(crate) height: u32,
    pub(crate) scale_factor: f64,
    pub(crate) primary: bool,
    pub(crate) details: Vec<Detail>,
}

#[derive(Serialize)]
pub struct HardwareInfo {
    pub(crate) computer_name: String,
    pub(crate) computer_details: Vec<Detail>,
    pub(crate) connection: Option<crate::connection::Connection>,
    pub(crate) peripherals: Vec<Peripheral>,
    pub(crate) displays: Vec<Display>,
}
