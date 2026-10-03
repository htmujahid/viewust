pub mod commands;
mod computer;
mod displays;
mod edid;
mod jacks;
mod model;
mod usb;

pub use model::Peripheral;

use crate::common::Detail;

pub fn peripherals() -> Vec<Peripheral> {
    let mut all = usb::list();
    all.extend(jacks::audio_jacks());
    all.sort_by(|a, b| a.kind.cmp(b.kind).then_with(|| a.name.cmp(&b.name)));
    all
}

pub fn computer_details() -> Vec<Detail> {
    computer::details()
}
