//! Human-readable facts about each device, shown when it is clicked.

use nusb::DeviceInfo;
use serde::Serialize;
use sysinfo::System;

/// One row in the details panel. Rows with the same `section` are grouped.
#[derive(Serialize, Clone)]
pub struct Detail {
    section: String,
    label: String,
    value: String,
}

pub struct Details(Vec<Detail>);

impl Details {
    pub fn new() -> Self {
        Self(Vec::new())
    }

    pub fn add(&mut self, section: impl Into<String>, label: impl Into<String>, value: impl Into<String>) {
        let value = value.into();
        if !value.trim().is_empty() {
            self.0.push(Detail {
                section: section.into(),
                label: label.into(),
                value,
            });
        }
    }

    pub fn add_opt(&mut self, section: impl Into<String>, label: &str, value: Option<impl Into<String>>) {
        if let Some(v) = value {
            self.add(section, label, v);
        }
    }

    pub fn finish(self) -> Vec<Detail> {
        self.0
    }
}

pub fn format_bytes(bytes: u64) -> String {
    const UNITS: [&str; 5] = ["B", "KiB", "MiB", "GiB", "TiB"];
    let mut value = bytes as f64;
    let mut unit = 0;
    while value >= 1024.0 && unit < UNITS.len() - 1 {
        value /= 1024.0;
        unit += 1;
    }
    if unit == 0 {
        format!("{bytes} B")
    } else {
        format!("{value:.1} {}", UNITS[unit])
    }
}

pub fn read(path: impl AsRef<std::path::Path>) -> Option<String> {
    std::fs::read_to_string(path)
        .ok()
        .map(|s| s.trim().to_owned())
        .filter(|s| !s.is_empty())
}

// ---------------------------------------------------------------- computer

pub fn computer_details() -> Vec<Detail> {
    let mut sys = System::new();
    sys.refresh_memory();
    sys.refresh_cpu_all();

    let mut d = Details::new();

    #[cfg(target_os = "linux")]
    {
        let dmi = |f: &str| read(format!("/sys/class/dmi/id/{f}"));
        d.add_opt("System", "Manufacturer", dmi("sys_vendor"));
        d.add_opt("System", "Model", dmi("product_name"));
        d.add_opt("System", "Motherboard", dmi("board_name"));
        d.add_opt(
            "System",
            "BIOS",
            dmi("bios_vendor").map(|v| match dmi("bios_version") {
                Some(ver) => format!("{v} {ver}"),
                None => v,
            }),
        );
    }

    d.add_opt("Operating system", "Host name", System::host_name());
    d.add_opt("Operating system", "System", System::long_os_version());
    d.add_opt("Operating system", "Kernel", System::kernel_version());
    d.add("Operating system", "Architecture", System::cpu_arch());
    let up = System::uptime();
    d.add(
        "Operating system",
        "Uptime",
        format!("{}d {}h {}m", up / 86400, (up % 86400) / 3600, (up % 3600) / 60),
    );

    if let Some(cpu) = sys.cpus().first() {
        d.add("Processor", "Model", cpu.brand().trim());
        d.add("Processor", "Vendor", cpu.vendor_id());
    }
    d.add_opt(
        "Processor",
        "Physical cores",
        System::physical_core_count().map(|n| n.to_string()),
    );
    d.add("Processor", "Logical cores", sys.cpus().len().to_string());

    d.add("Memory", "Installed", format_bytes(sys.total_memory()));
    d.add("Memory", "In use", format_bytes(sys.used_memory()));
    if sys.total_swap() > 0 {
        d.add("Memory", "Swap", format_bytes(sys.total_swap()));
    }
    d.finish()
}

// --------------------------------------------------------------------- usb

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

fn speed(speed: Option<nusb::Speed>) -> Option<&'static str> {
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

    d.add("Identification", "Vendor ID", format!("0x{:04x}", device.vendor_id()));
    d.add("Identification", "Product ID", format!("0x{:04x}", device.product_id()));
    d.add("Identification", "USB version", bcd(device.usb_version()));
    d.add("Identification", "Device version", bcd(device.device_version()));

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
        d.add_opt("Power", "Configurations", read(sys.join("bNumConfigurations")));
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
        d.add("Interfaces", format!("Interface {}", i.interface_number()), parts.join(" · "));
    }
    d.finish()
}

// ----------------------------------------------------------------- display

pub struct Edid {
    manufacturer: String,
    model: Option<String>,
    serial: Option<String>,
    product_code: u16,
    week: u8,
    year: u16,
    width_cm: u8,
    height_cm: u8,
}

impl Edid {
    pub fn model(&self) -> Option<&str> {
        self.model.as_deref()
    }

    /// Physical panel width in centimetres, when the monitor reports it.
    pub fn width_cm(&self) -> Option<u32> {
        (self.width_cm > 0).then_some(self.width_cm as u32)
    }
}

fn maker_name(code: &str) -> Option<&'static str> {
    Some(match code {
        "ACI" | "AUS" => "ASUS",
        "ACR" => "Acer",
        "AOC" => "AOC",
        "BNQ" => "BenQ",
        "DEL" => "Dell",
        "GBT" => "Gigabyte",
        "GSM" => "LG",
        "HPN" | "HWP" => "HP",
        "LEN" => "Lenovo",
        "MSI" => "MSI",
        "PHL" => "Philips",
        "SAM" => "Samsung",
        "SNY" => "Sony",
        "VSC" => "ViewSonic",
        "XMI" => "Xiaomi",
        _ => return None,
    })
}

fn parse_edid(data: &[u8]) -> Option<Edid> {
    if data.len() < 128 || data[0..8] != [0, 255, 255, 255, 255, 255, 255, 0] {
        return None;
    }
    let id = u16::from_be_bytes([data[8], data[9]]);
    let letter = |v: u16| (b'A' + (v as u8 & 0x1f) - 1) as char;
    let manufacturer: String = [letter(id >> 10), letter(id >> 5), letter(id)].iter().collect();

    let mut model = None;
    let mut serial = None;
    for block in data[54..126].chunks(18) {
        if block[0..3] == [0, 0, 0] {
            let text = String::from_utf8_lossy(&block[5..18])
                .split('\n')
                .next()
                .unwrap_or_default()
                .trim()
                .to_owned();
            match block[3] {
                0xfc => model = Some(text),
                0xff => serial = Some(text),
                _ => {}
            }
        }
    }

    Some(Edid {
        manufacturer,
        model,
        serial,
        product_code: u16::from_le_bytes([data[10], data[11]]),
        week: data[16],
        year: 1990 + data[17] as u16,
        width_cm: data[21],
        height_cm: data[22],
    })
}

/// EDID blocks of every connected display, with the connector they're on.
#[cfg(target_os = "linux")]
pub fn connected_edids() -> Vec<(String, Edid)> {
    let mut out = Vec::new();
    let Ok(entries) = std::fs::read_dir("/sys/class/drm") else {
        return out;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        // card0-HDMI-A-1 → HDMI-A-1
        let file = entry.file_name().to_string_lossy().into_owned();
        let Some((_, connector)) = file.split_once('-') else {
            continue;
        };
        if read(path.join("status")).as_deref() != Some("connected") {
            continue;
        }
        if let Some(edid) = std::fs::read(path.join("edid")).ok().and_then(|b| parse_edid(&b)) {
            out.push((connector.to_owned(), edid));
        }
    }
    out.sort_by(|a, b| a.0.cmp(&b.0));
    out
}

#[cfg(not(target_os = "linux"))]
pub fn connected_edids() -> Vec<(String, Edid)> {
    Vec::new()
}

pub struct DisplayFacts<'a> {
    pub width: u32,
    pub height: u32,
    pub x: i32,
    pub y: i32,
    pub scale_factor: f64,
    pub primary: bool,
    pub connector: Option<&'a str>,
    pub edid: Option<&'a Edid>,
}

pub fn display_details(f: &DisplayFacts) -> Vec<Detail> {
    let mut d = Details::new();

    if let Some(e) = f.edid {
        d.add_opt("Monitor", "Model", e.model.clone());
        let maker = maker_name(&e.manufacturer)
            .map(|n| format!("{n} ({})", e.manufacturer))
            .unwrap_or_else(|| e.manufacturer.clone());
        d.add("Monitor", "Manufacturer", maker);
        d.add_opt("Monitor", "Serial number", e.serial.clone());
        d.add("Monitor", "Product code", format!("0x{:04x}", e.product_code));
        if e.year > 1990 {
            d.add("Monitor", "Manufactured", format!("Week {}, {}", e.week, e.year));
        }
        if e.width_cm > 0 && e.height_cm > 0 {
            let (w, h) = (e.width_cm as f64, e.height_cm as f64);
            d.add(
                "Panel",
                "Physical size",
                format!("{} × {} cm · {:.1}″ diagonal", e.width_cm, e.height_cm, (w * w + h * h).sqrt() / 2.54),
            );
            d.add(
                "Panel",
                "Pixel density",
                format!("{:.0} PPI", f.width as f64 / (w / 2.54)),
            );
        }
    }

    d.add("Display", "Resolution", format!("{} × {}", f.width, f.height));
    d.add("Display", "Scale factor", format!("{}×", f.scale_factor));
    d.add("Display", "Position", format!("{}, {}", f.x, f.y));
    d.add("Display", "Primary", if f.primary { "Yes" } else { "No" });
    d.add_opt("Connection", "Connector", f.connector);
    d.finish()
}
