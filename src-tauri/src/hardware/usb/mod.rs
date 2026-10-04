use nusb::{DeviceInfo, MaybeFuture};
mod classify;
mod facts;

use super::model::Peripheral;
use classify::classify;

fn speed_name(speed: Option<nusb::Speed>) -> String {
    match speed {
        Some(nusb::Speed::Low) => "USB 1.0",
        Some(nusb::Speed::Full) => "USB 1.1",
        Some(nusb::Speed::High) => "USB 2.0",
        Some(nusb::Speed::Super) => "USB 3.0",
        Some(nusb::Speed::SuperPlus) => "USB 3.1+",
        _ => "USB",
    }
    .into()
}

fn is_internal(device: &DeviceInfo) -> bool {
    std::fs::read_to_string(device.sysfs_path().join("removable"))
        .is_ok_and(|v| v.trim() == "fixed")
}

fn is_receiver(device: &DeviceInfo, kinds: &[&str]) -> bool {
    let name = format!(
        "{} {}",
        device.manufacturer_string().unwrap_or_default(),
        device.product_string().unwrap_or_default()
    )
    .to_lowercase();
    let carries_input = kinds.iter().all(|k| {
        matches!(
            *k,
            "keyboard" | "mouse" | "gamepad" | "audio" | "microphone"
        )
    });
    let both_inputs = kinds.contains(&"keyboard") && kinds.contains(&"mouse");
    let says_receiver = [
        "receiver",
        "unifying",
        "dongle",
        "2.4g",
        "wireless",
        "bluetooth",
    ]
    .iter()
    .any(|w| name.contains(w));
    carries_input && (both_inputs || says_receiver)
}

fn is_root_hub(device: &DeviceInfo) -> bool {
    device.vendor_id() == 0x1d6b && device.class() == 0x09
}

pub(crate) fn list() -> Vec<Peripheral> {
    let Ok(devices) = nusb::list_devices().wait() else {
        return Vec::new();
    };

    let mut out: Vec<Peripheral> = Vec::new();
    for d in devices.filter(|d| !is_root_hub(d) && !is_internal(d)) {
        let id = format!("{}-{}", d.bus_id(), d.device_address());
        let kinds = classify(&d);
        let speed = speed_name(d.speed());
        let receiver = is_receiver(&d, &kinds);

        let make = |id: String, kind: &'static str, via: Option<String>| Peripheral {
            id,
            wireless: via.is_some(),
            via,
            name: d.product_string().map(str::to_owned).unwrap_or_else(|| {
                format!("USB device {:04x}:{:04x}", d.vendor_id(), d.product_id())
            }),
            manufacturer: d.manufacturer_string().map(str::to_owned),
            kind,
            connection: speed.clone(),
            vendor_id: format!("{:04x}", d.vendor_id()),
            product_id: format!("{:04x}", d.product_id()),
            serial_number: d.serial_number().map(str::to_owned),
            details: facts::usb_details(&d),
        };

        if receiver {
            out.push(make(id.clone(), "wireless", None));
            for kind in kinds {
                out.push(make(format!("{id}:{kind}"), kind, Some(id.clone())));
            }
        } else {
            for kind in kinds {
                out.push(make(format!("{id}:{kind}"), kind, None));
            }
        }
    }
    out
}
