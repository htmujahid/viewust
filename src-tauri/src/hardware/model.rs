//! What the devices map shows.

use serde::Serialize;

use crate::common::Detail;

#[derive(Serialize)]
pub struct Peripheral {
    pub(crate) id: String,
    /// Id of the receiver this device talks to wirelessly, if any.
    pub(crate) via: Option<String>,
    /// True when the link to its parent is radio rather than a cable.
    pub(crate) wireless: bool,
    pub(crate) name: String,
    pub(crate) manufacturer: Option<String>,
    /// Drives which illustration the UI draws.
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
    /// Connector it is plugged into ("DP-3"), when known.
    pub(crate) connector: Option<String>,
    /// Real panel width, used to draw the monitor at its true relative size.
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
