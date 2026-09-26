//! The USB facts shown for a device in the sidebar.

use crate::common::sysfs::*;
use crate::common::{Detail, Details};
use nusb::DeviceInfo;

fn class_name(class: u8) -> &'static str {
    match class {
        0x00 => "Defined per interface",
        0x01 => "Audio",
        0x02 => "Communications",
        0x03 => "Human interface (HID)",
        0x05 => "Physical",
        0x06 => "Imaging",
        0x07 => "Printer",
        0x08 => "Mass storage",
        0x09 => "Hub",
        0x0a => "CDC data",
        0x0b => "Smart card",
        0x0d => "Content security",
        0x0e => "Video",
        0x0f => "Personal healthcare",
        0x10 => "Audio/video",
        0xdc => "Diagnostic",
        0xe0 => "Wireless controller",
        0xef => "Miscellaneous",
        0xfe => "Application specific",
        0xff => "Vendor specific",
        _ => "Unknown",
    }
}

fn bcd(v: u16) -> String {
    format!("{:x}.{:02x}", v >> 8, v & 0xff)
}

pub(crate) fn speed(speed: Option<nusb::Speed>) -> Option<&'static str> {
    Some(match speed? {
        nusb::Speed::Low => "Low speed · 1.5 Mbit/s",
        nusb::Speed::Full => "Full speed · 12 Mbit/s",
        nusb::Speed::High => "High speed · 480 Mbit/s",
        nusb::Speed::Super => "SuperSpeed · 5 Gbit/s",
        nusb::Speed::SuperPlus => "SuperSpeed+ · 10 Gbit/s",
        _ => return None,
    })
}

pub fn usb_details(device: &DeviceInfo) -> Vec<Detail> {
    let mut d = Details::new();

    d.add_opt("Device", "Product", device.product_string());
    d.add_opt("Device", "Manufacturer", device.manufacturer_string());
    d.add_opt("Device", "Serial number", device.serial_number());

    d.add(
        "Identification",
        "Vendor ID",
        format!("0x{:04x}", device.vendor_id()),
    );
    d.add(
        "Identification",
        "Product ID",
        format!("0x{:04x}", device.product_id()),
    );
    d.add("Identification", "USB version", bcd(device.usb_version()));
    d.add(
        "Identification",
        "Device version",
        bcd(device.device_version()),
    );

    d.add_opt("Connection", "Speed", speed(device.speed()));
    d.add("Connection", "Bus", device.bus_id());
    d.add("Connection", "Address", device.device_address().to_string());

    #[cfg(target_os = "linux")]
    {
        let sys = device.sysfs_path();
        d.add_opt("Connection", "Port", read(sys.join("devpath")));
        d.add_opt(
            "Connection",
            "Removable",
            read(sys.join("removable")).map(|v| match v.as_str() {
                "removable" => "Yes".to_owned(),
                "fixed" => "No, built in".to_owned(),
                _ => "Unknown".to_owned(),
            }),
        );
        d.add_opt("Power", "Maximum draw", read(sys.join("bMaxPower")));
        d.add_opt(
            "Power",
            "Configurations",
            read(sys.join("bNumConfigurations")),
        );
    }

    d.add(
        "Class",
        "Device class",
        format!("{} (0x{:02x})", class_name(device.class()), device.class()),
    );
    d.add(
        "Class",
        "Subclass / protocol",
        format!("0x{:02x} / 0x{:02x}", device.subclass(), device.protocol()),
    );

    for i in device.interfaces() {
        let mut parts = vec![format!("{} (0x{:02x})", class_name(i.class()), i.class())];
        if let Some(name) = i.interface_string() {
            parts.push(name.to_owned());
        }
        #[cfg(target_os = "linux")]
        {
            // /sys/.../1-9  →  /sys/.../1-9/1-9:1.0/driver → ".../uvcvideo"
            let sys = device.sysfs_path();
            let dir = format!(
                "{}:1.{}",
                sys.file_name().unwrap_or_default().to_string_lossy(),
                i.interface_number()
            );
            let driver = std::fs::read_link(sys.join(dir).join("driver"))
                .ok()
                .and_then(|p| p.file_name().map(|n| n.to_string_lossy().into_owned()));
            if let Some(driver) = driver {
                parts.push(format!("driver {driver}"));
            }
        }
        d.add(
            "Interfaces",
            format!("Interface {}", i.interface_number()),
            parts.join(" · "),
        );
    }
    d.finish()
}
