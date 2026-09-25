//! Deep, per-device technical report shown on a device's own page.
//!
//! Everything here is read from the operating system (sysfs, procfs, the
//! kernel's USB/HID descriptors and the system's hardware-ID lists). Devices
//! don't report their internal parts (sensor, microcontroller), only what
//! these interfaces expose.

use crate::detail::{format_bytes, read, Detail, Details};
use std::path::{Path, PathBuf};
use std::process::Command;

#[tauri::command]
pub fn device_report(id: String) -> Vec<Detail> {
    let mut d = Details::new();
    #[cfg(target_os = "linux")]
    {
        if id == "computer" {
            computer(&mut d);
        } else if let Some(connector) = id.strip_prefix("display:") {
            display(&mut d, connector);
        } else if id.starts_with("net:") {
            // the router and internet cards carry all their detail already
        } else if let Some(rest) = id.strip_prefix("sys:") {
            crate::system::deep(&mut d, rest);
        } else if let Some(rest) = id.strip_prefix("jack-") {
            jack(&mut d, rest);
        } else {
            usb(&mut d, &id);
        }
    }
    #[cfg(not(target_os = "linux"))]
    {
        let _ = &id;
        d.add(
            "Technical",
            "Availability",
            "Low-level details are only available on Linux for now",
        );
    }
    d.finish()
}

// ------------------------------------------------------------ id databases

/// Looks up `vendor` (and optionally `device`) in a `usb.ids` / `pci.ids` file.
fn lookup_ids(files: &[&str], vendor: &str, device: Option<&str>) -> (Option<String>, Option<String>) {
    let Some(text) = files.iter().find_map(|f| std::fs::read_to_string(f).ok()) else {
        return (None, None);
    };
    lookup_text(&text, vendor, device)
}

/// Same lookup over text that has already been read (avoids re-reading the
/// 1 MB `pci.ids` file for every device).
pub(crate) fn lookup_text(text: &str, vendor: &str, device: Option<&str>) -> (Option<String>, Option<String>) {
    let (vendor, device) = (vendor.to_lowercase(), device.map(str::to_lowercase));
    let mut vendor_name = None;
    let mut device_name = None;
    for line in text.lines() {
        if line.starts_with('#') || line.trim().is_empty() {
            continue;
        }
        if !line.starts_with('\t') {
            if vendor_name.is_some() {
                break; // left the vendor's block
            }
            if let Some((id, name)) = line.split_once("  ") {
                if id.to_lowercase() == vendor {
                    vendor_name = Some(name.trim().to_owned());
                }
            }
        } else if vendor_name.is_some() && !line.starts_with("\t\t") {
            if let (Some(want), Some((id, name))) = (&device, line.trim_start().split_once("  ")) {
                if id.to_lowercase() == *want {
                    device_name = Some(name.trim().to_owned());
                    break;
                }
            }
        }
    }
    (vendor_name, device_name)
}

const USB_IDS: &[&str] = &["/usr/share/hwdata/usb.ids", "/usr/share/misc/usb.ids", "/var/lib/usbutils/usb.ids"];
pub(crate) const PCI_IDS: &[&str] = &["/usr/share/hwdata/pci.ids", "/usr/share/misc/pci.ids"];

pub fn pnp_vendor(code: &str) -> Option<String> {
    let text = std::fs::read_to_string("/usr/share/hwdata/pnp.ids").ok()?;
    text.lines().find_map(|l| {
        let (id, name) = l.split_once('\t')?;
        (id == code).then(|| name.trim().to_owned())
    })
}

// ------------------------------------------------------------- bit helpers

/// Parses a sysfs bitmap such as `1f0000 0 0` (most significant word first).
fn bitmap(text: &str) -> Vec<usize> {
    let width = usize::BITS as usize;
    let mut out = Vec::new();
    for (i, word) in text.split_whitespace().rev().enumerate() {
        if let Ok(v) = u64::from_str_radix(word, 16) {
            for b in 0..width.min(64) {
                if v >> b & 1 == 1 {
                    out.push(i * width + b);
                }
            }
        }
    }
    out
}

fn named(bits: &[usize], name: impl Fn(usize) -> Option<&'static str>) -> Vec<&'static str> {
    bits.iter().filter_map(|b| name(*b)).collect()
}

fn ev_name(c: usize) -> Option<&'static str> {
    Some(match c {
        0x00 => "Sync",
        0x01 => "Keys / buttons",
        0x02 => "Relative motion",
        0x03 => "Absolute position",
        0x04 => "Scan codes",
        0x05 => "Switches",
        0x11 => "LEDs",
        0x12 => "Sound",
        0x14 => "Key repeat",
        0x15 => "Force feedback",
        _ => return None,
    })
}

fn rel_name(c: usize) -> Option<&'static str> {
    Some(match c {
        0x00 => "X",
        0x01 => "Y",
        0x02 => "Z",
        0x06 => "Horizontal wheel",
        0x07 => "Dial",
        0x08 => "Wheel",
        0x09 => "Misc",
        0x0b => "Wheel (high resolution)",
        0x0c => "Horizontal wheel (high resolution)",
        _ => return None,
    })
}

fn abs_name(c: usize) -> Option<&'static str> {
    Some(match c {
        0x00 => "X",
        0x01 => "Y",
        0x02 => "Z",
        0x03 => "Rx",
        0x04 => "Ry",
        0x05 => "Rz",
        0x10 => "Hat 0",
        0x18 => "Pressure",
        0x2f => "Multitouch slot",
        0x35 => "Touch X",
        0x36 => "Touch Y",
        _ => return None,
    })
}

fn button_name(c: usize) -> Option<&'static str> {
    Some(match c {
        0x110 => "Left",
        0x111 => "Right",
        0x112 => "Middle",
        0x113 => "Side",
        0x114 => "Extra",
        0x115 => "Forward",
        0x116 => "Back",
        0x117 => "Task",
        0x130 => "A / South",
        0x131 => "B / East",
        0x133 => "X / North",
        0x134 => "Y / West",
        0x136 => "Left bumper",
        0x137 => "Right bumper",
        0x13a => "Select",
        0x13b => "Start",
        0x13c => "Mode",
        0x13d => "Left stick",
        0x13e => "Right stick",
        _ => return None,
    })
}

fn led_name(c: usize) -> Option<&'static str> {
    Some(match c {
        0 => "Num Lock",
        1 => "Caps Lock",
        2 => "Scroll Lock",
        3 => "Compose",
        4 => "Kana",
        _ => return None,
    })
}

// ------------------------------------------------------------------ driver

fn modinfo(module: &str, field: &str) -> Vec<String> {
    Command::new("modinfo")
        .args(["-F", field, module])
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| {
            String::from_utf8_lossy(&o.stdout)
                .lines()
                .map(|l| l.trim().to_owned())
                .filter(|l| !l.is_empty())
                .collect()
        })
        .unwrap_or_default()
}

pub(crate) fn driver_rows(d: &mut Details, section: &str, driver: &str) {
    d.add(section, "Driver", driver);
    let description = modinfo(driver, "description");
    if description.is_empty() {
        d.add(section, "Module", "Built into the kernel (or not a loadable module)");
        return;
    }
    d.add(section, "Description", description.join(" "));
    let authors = modinfo(driver, "author");
    if !authors.is_empty() {
        d.add(section, "Author", authors.join(", "));
    }
    d.add_opt(section, "License", modinfo(driver, "license").into_iter().next());
    d.add_opt(section, "Version", modinfo(driver, "version").into_iter().next());
    d.add_opt(section, "Module file", modinfo(driver, "filename").into_iter().next());
    let depends = modinfo(driver, "depends");
    if !depends.is_empty() && !depends.join("").is_empty() {
        d.add(section, "Depends on", depends.join(", "));
    }
    let used = read(format!("/sys/module/{}/refcnt", driver.replace('-', "_")));
    d.add_opt(section, "Users", used.map(|n| format!("{n} reference(s)")));
}

// --------------------------------------------------------------------- USB

struct Endpoint {
    interface: u8,
    alt: u8,
    address: u8,
    attributes: u8,
    max_packet: u16,
    interval: u8,
}

#[derive(Default)]
struct Descriptors {
    config_attributes: Option<u8>,
    endpoints: Vec<Endpoint>,
    /// interface number → (HID version, country code, report descriptor length)
    hid: std::collections::HashMap<u8, (u16, u8, u16)>,
}

fn parse_descriptors(raw: &[u8]) -> Descriptors {
    let mut out = Descriptors::default();
    let (mut interface, mut alt) = (0u8, 0u8);
    let mut o = 0;
    while o + 2 <= raw.len() {
        let len = raw[o] as usize;
        if len < 2 || o + len > raw.len() {
            break;
        }
        let b = &raw[o..o + len];
        match b[1] {
            0x02 if len >= 8 => out.config_attributes = Some(b[7]),
            0x04 if len >= 9 => {
                interface = b[2];
                alt = b[3];
            }
            0x05 if len >= 7 => out.endpoints.push(Endpoint {
                interface,
                alt,
                address: b[2],
                attributes: b[3],
                max_packet: u16::from_le_bytes([b[4], b[5]]) & 0x7ff,
                interval: b[6],
            }),
            0x21 if len >= 9 => {
                let version = u16::from_le_bytes([b[2], b[3]]);
                let report_len = u16::from_le_bytes([b[7], b[8]]);
                out.hid.insert(interface, (version, b[4], report_len));
            }
            _ => {}
        }
        o += len;
    }
    out
}

fn endpoint_text(e: &Endpoint, speed_mbps: f64) -> String {
    let kind = ["Control", "Isochronous", "Bulk", "Interrupt"][(e.attributes & 3) as usize];
    let dir = if e.address & 0x80 != 0 { "IN (to computer)" } else { "OUT (to device)" };
    let mut text = format!("{dir} · {kind} · {} bytes/packet", e.max_packet);
    if matches!(e.attributes & 3, 1 | 3) && e.interval > 0 {
        // Low/full speed count milliseconds; high speed counts 125 µs steps.
        let micros = if speed_mbps >= 480.0 {
            125.0 * 2f64.powi(e.interval as i32 - 1)
        } else if e.attributes & 3 == 1 {
            1000.0 * 2f64.powi(e.interval as i32 - 1)
        } else {
            1000.0 * e.interval as f64
        };
        text += &format!(" · polled every {} ({:.0} Hz)", fmt_micros(micros), 1_000_000.0 / micros);
    }
    text
}

fn fmt_micros(us: f64) -> String {
    if us >= 1000.0 {
        format!("{} ms", us / 1000.0)
    } else {
        format!("{us} µs")
    }
}

struct Iface {
    path: PathBuf,
    number: u8,
    class: u8,
    protocol: u8,
}

fn interfaces(sys: &Path) -> Vec<Iface> {
    let base = sys.file_name().unwrap_or_default().to_string_lossy().into_owned();
    let mut out: Vec<Iface> = std::fs::read_dir(sys)
        .into_iter()
        .flatten()
        .flatten()
        .filter(|e| e.file_name().to_string_lossy().starts_with(&format!("{base}:")))
        .filter_map(|e| {
            let path = e.path();
            let hex = |f: &str| read(path.join(f)).and_then(|v| u8::from_str_radix(&v, 16).ok());
            Some(Iface {
                number: hex("bInterfaceNumber")?,
                class: hex("bInterfaceClass").unwrap_or(0),
                protocol: hex("bInterfaceProtocol").unwrap_or(0),
                path,
            })
        })
        .collect();
    out.sort_by_key(|i| i.number);
    out
}

/// A receiver's keyboard and mouse live on different interfaces; show only the
/// ones belonging to the node that was opened.
fn wanted(kind: Option<&str>, i: &Iface) -> bool {
    match kind {
        Some("keyboard") => !(i.class == 3 && i.protocol == 2),
        Some("mouse") => i.class == 3 && i.protocol == 2,
        _ => true,
    }
}

#[cfg(target_os = "linux")]
fn usb(d: &mut Details, id: &str) {
    use nusb::MaybeFuture;

    let (base, kind) = match id.split_once(':') {
        Some((b, k)) => (b, Some(k)),
        None => (id, None),
    };
    let Some(dev) = nusb::list_devices()
        .wait()
        .ok()
        .and_then(|mut it| it.find(|x| format!("{}-{}", x.bus_id(), x.device_address()) == base))
    else {
        d.add("Technical", "Status", "This device is no longer connected");
        return;
    };
    let sys = dev.sysfs_path().to_path_buf();

    // Who made the chip, per the public USB ID registry.
    let (vendor, product) = lookup_ids(
        USB_IDS,
        &format!("{:04x}", dev.vendor_id()),
        Some(&format!("{:04x}", dev.product_id())),
    );
    d.add_opt("Registry", "Vendor ID owner", vendor);
    d.add_opt("Registry", "Registered product", product);
    d.add(
        "Registry",
        "Note",
        "The vendor is the company that registered this USB ID. It usually names the chip \
         or module maker, not the sensor or microcontroller model, which devices never report.",
    );

    let speed_mbps = read(sys.join("speed")).and_then(|s| s.parse::<f64>().ok()).unwrap_or(12.0);
    let desc = std::fs::read(sys.join("descriptors")).map(|r| parse_descriptors(&r)).unwrap_or_default();

    if let Some(a) = desc.config_attributes {
        let mut parts = vec![if a & 0x40 != 0 { "Self-powered" } else { "Bus-powered" }];
        if a & 0x20 != 0 {
            parts.push("remote wake-up (can wake the computer)");
        }
        d.add("Power", "Configuration", parts.join(" · "));
    }

    let ifaces: Vec<Iface> = interfaces(&sys).into_iter().filter(|i| wanted(kind, i)).collect();

    // Several interfaces often share one driver; describe each driver once.
    let mut drivers: Vec<(String, Vec<u8>)> = Vec::new();
    for iface in &ifaces {
        let driver = std::fs::read_link(iface.path.join("driver"))
            .ok()
            .and_then(|p| p.file_name().map(|f| f.to_string_lossy().into_owned()));
        if let Some(driver) = driver {
            match drivers.iter_mut().find(|(name, _)| *name == driver) {
                Some((_, list)) => list.push(iface.number),
                None => drivers.push((driver, vec![iface.number])),
            }
        }
    }
    for (driver, numbers) in &drivers {
        let section = format!("Driver · {driver}");
        driver_rows(d, &section, driver);
        d.add(
            &section,
            "Used by interfaces",
            numbers.iter().map(u8::to_string).collect::<Vec<_>>().join(", "),
        );
    }

    for iface in &ifaces {
        let n = iface.number;

        let subdirs: Vec<(String, PathBuf)> = std::fs::read_dir(&iface.path)
            .into_iter()
            .flatten()
            .flatten()
            .map(|e| (e.file_name().to_string_lossy().into_owned(), e.path()))
            .collect();

        for (name, path) in &subdirs {
            // HID interface, e.g. 0003:2717:503F.0002
            if name.matches(':').count() == 2 && path.join("report_descriptor").exists() {
                hid_section(d, n, path, desc.hid.get(&n).copied(), &dev);
            }
            if name == "video4linux" {
                for e in std::fs::read_dir(path).into_iter().flatten().flatten() {
                    let node = e.file_name().to_string_lossy().into_owned();
                    let label = read(e.path().join("name")).unwrap_or_default();
                    d.add(format!("Interface {n} · video"), format!("/dev/{node}"), label);
                }
            }
            if name == "sound" {
                for e in std::fs::read_dir(path).into_iter().flatten().flatten() {
                    if let Some(card) = e.file_name().to_string_lossy().strip_prefix("card") {
                        if let Ok(card) = card.parse::<u32>() {
                            audio_streams(d, n, card);
                        }
                    }
                }
            }
        }
    }

    // Endpoints, trimmed to the interfaces we're showing.
    let shown: Vec<u8> = ifaces.iter().map(|i| i.number).collect();
    let eps: Vec<&Endpoint> = desc.endpoints.iter().filter(|e| shown.contains(&e.interface)).collect();
    for e in eps.iter().take(40) {
        let alt = if e.alt > 0 { format!(" alt {}", e.alt) } else { String::new() };
        d.add(
            "Endpoints",
            format!("Interface {}{alt} · 0x{:02x}", e.interface, e.address),
            endpoint_text(e, speed_mbps),
        );
    }
    if eps.len() > 40 {
        d.add("Endpoints", "…", format!("{} more endpoints not shown", eps.len() - 40));
    }
}

#[cfg(target_os = "linux")]
fn audio_streams(d: &mut Details, iface: u8, card: u32) {
    let section = format!("Interface {iface} · audio");
    d.add_opt(&section, "Card", read(format!("/proc/asound/card{card}/id")));
    for n in 0..4 {
        let Ok(text) = std::fs::read_to_string(format!("/proc/asound/card{card}/stream{n}")) else {
            break;
        };
        let mut direction = "";
        let (mut formats, mut channels, mut rates): (Vec<String>, Vec<u32>, Vec<u32>) = Default::default();
        let flush = |d: &mut Details, direction: &str, f: &mut Vec<String>, c: &mut Vec<u32>, r: &mut Vec<u32>| {
            if direction.is_empty() || f.is_empty() {
                return;
            }
            f.sort();
            f.dedup();
            c.sort();
            c.dedup();
            r.sort();
            r.dedup();
            let chans = match (c.first(), c.last()) {
                (Some(a), Some(b)) if a != b => format!("{a}–{b} channels"),
                (Some(a), _) => format!("{a} channel{}", if *a == 1 { "" } else { "s" }),
                _ => String::new(),
            };
            let hz = match (r.first(), r.last()) {
                (Some(a), Some(b)) if a != b => format!("{a}–{b} Hz"),
                (Some(a), _) => format!("{a} Hz"),
                _ => String::new(),
            };
            d.add(
                format!("Interface {iface} · audio"),
                direction.trim_end_matches(':').to_owned(),
                [f.join(", "), chans, hz].into_iter().filter(|s| !s.is_empty()).collect::<Vec<_>>().join(" · "),
            );
            f.clear();
            c.clear();
            r.clear();
        };
        for line in text.lines().map(str::trim) {
            if line == "Capture:" || line == "Playback:" {
                flush(d, direction, &mut formats, &mut channels, &mut rates);
                direction = if line == "Capture:" { "Recording" } else { "Playback" };
            } else if let Some(v) = line.strip_prefix("Format:") {
                formats.push(v.trim().to_owned());
            } else if let Some(v) = line.strip_prefix("Channels:") {
                channels.extend(v.split(|c: char| !c.is_ascii_digit()).filter_map(|x| x.parse::<u32>().ok()));
            } else if let Some(v) = line.strip_prefix("Rates:") {
                rates.extend(v.split(|c: char| !c.is_ascii_digit()).filter_map(|x| x.parse::<u32>().ok()));
            }
        }
        flush(d, direction, &mut formats, &mut channels, &mut rates);
    }
    if let Ok(out) = Command::new("amixer").args(["-c", &card.to_string(), "scontrols"]).output() {
        let names: Vec<String> = String::from_utf8_lossy(&out.stdout)
            .lines()
            .filter_map(|l| l.split('\'').nth(1).map(str::to_owned))
            .collect();
        if !names.is_empty() {
            d.add(&section, "Mixer controls", names.join(", "));
        }
    }
}

// --------------------------------------------------------------------- HID

#[cfg(target_os = "linux")]
fn hid_section(d: &mut Details, iface: u8, hid_dir: &Path, hid: Option<(u16, u8, u16)>, dev: &nusb::DeviceInfo) {
    let name = |what: &str| format!("Interface {iface} · {what}");
    let product = dev.product_string().unwrap_or_default();
    let maker = dev.manufacturer_string().unwrap_or_default();

    // The kernel's input devices created for this interface.
    for e in std::fs::read_dir(hid_dir.join("input")).into_iter().flatten().flatten() {
        let p = e.path();
        let full = read(p.join("name")).unwrap_or_default();
        let short = full
            .strip_prefix(&format!("{maker} {product}"))
            .or_else(|| full.strip_prefix(product))
            .map(|s| s.trim().to_owned())
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| "Main".into());
        let section = name(&format!("input · {short}"));
        d.add(&section, "Kernel name", full);

        let handlers: Vec<String> = std::fs::read_dir(&p)
            .into_iter()
            .flatten()
            .flatten()
            .map(|x| x.file_name().to_string_lossy().into_owned())
            .filter(|n| n.starts_with("event") || n.starts_with("mouse") || n.starts_with("js"))
            .collect();
        d.add(&section, "Device nodes", handlers.iter().map(|h| format!("/dev/input/{h}")).collect::<Vec<_>>().join(", "));

        let cap = |f: &str| read(p.join("capabilities").join(f)).map(|t| bitmap(&t)).unwrap_or_default();
        let ev = cap("ev");
        d.add(&section, "Event types", named(&ev, ev_name).join(", "));

        let key = cap("key");
        let buttons = named(&key, button_name);
        if !buttons.is_empty() {
            d.add(&section, format!("Buttons ({})", buttons.len()), buttons.join(", "));
        }
        let keys = key.iter().filter(|k| **k < 0x100).count();
        if keys > 0 {
            d.add(&section, "Keyboard keys", format!("{keys} key codes"));
        }
        let extra = key.iter().filter(|k| **k >= 0x160 && button_name(**k).is_none()).count();
        if extra > 0 {
            d.add(&section, "Extra / media key codes", extra.to_string());
        }
        d.add(&section, "Movement axes", named(&cap("rel"), rel_name).join(", "));
        d.add(&section, "Absolute axes", named(&cap("abs"), abs_name).join(", "));
        d.add(&section, "Indicator LEDs", named(&cap("led"), led_name).join(", "));
    }

    // The raw HID report descriptor, decoded.
    let Ok(raw) = std::fs::read(hid_dir.join("report_descriptor")) else {
        return;
    };
    let section = name("HID report descriptor");
    if let Some((version, country, len)) = hid {
        d.add(&section, "HID version", format!("{:x}.{:02x}", version >> 8, version & 0xff));
        if country != 0 {
            d.add(&section, "Country code", country.to_string());
        }
        d.add(&section, "Descriptor length", format!("{len} bytes"));
    }
    for (label, value) in decode_hid(&raw) {
        d.add(&section, label, value);
    }
    let hex = raw
        .chunks(16)
        .map(|c| c.iter().map(|b| format!("{b:02x}")).collect::<Vec<_>>().join(" "))
        .collect::<Vec<_>>()
        .join("\n");
    d.add(&section, "Raw bytes", hex);
}

struct Field {
    page: u32,
    usages: Vec<u32>,
    size: u32,
    count: u32,
    min: i64,
    max: i64,
    phys_min: i64,
    phys_max: i64,
    unit: u32,
    exponent: i32,
    flags: u32,
    kind: &'static str,
    report_id: u32,
}

struct App {
    page: u32,
    usage: u32,
    fields: Vec<Field>,
}

fn usage_name(page: u32, usage: u32) -> String {
    match (page, usage) {
        (0x01, 0x01) => "Pointer".into(),
        (0x01, 0x02) => "Mouse".into(),
        (0x01, 0x04) => "Joystick".into(),
        (0x01, 0x05) => "Gamepad".into(),
        (0x01, 0x06) => "Keyboard".into(),
        (0x01, 0x07) => "Keypad".into(),
        (0x01, 0x30) => "X axis".into(),
        (0x01, 0x31) => "Y axis".into(),
        (0x01, 0x32) => "Z axis".into(),
        (0x01, 0x33) => "Rx axis".into(),
        (0x01, 0x34) => "Ry axis".into(),
        (0x01, 0x35) => "Rz axis".into(),
        (0x01, 0x36) => "Slider".into(),
        (0x01, 0x37) => "Dial".into(),
        (0x01, 0x38) => "Vertical wheel".into(),
        (0x01, 0x39) => "Hat switch".into(),
        (0x01, 0x48) => "Wheel resolution multiplier".into(),
        (0x01, 0x80) => "System control".into(),
        (0x01, 0x81) => "Power down".into(),
        (0x01, 0x82) => "Sleep".into(),
        (0x01, 0x83) => "Wake up".into(),
        (0x0c, 0x01) => "Consumer control".into(),
        (0x0c, 0x238) => "Horizontal wheel".into(),
        (0x0d, 0x04) => "Touch screen".into(),
        (0x0d, 0x05) => "Touch pad".into(),
        _ if page > 0x100 => format!("Vendor-defined ({page:#06x})"),
        _ => format!("Usage {page:#06x}:{usage:#x}"),
    }
}

fn signed(v: u32, size: usize) -> i64 {
    match size {
        1 => v as u8 as i8 as i64,
        2 => v as u16 as i16 as i64,
        _ => v as i32 as i64,
    }
}

fn parse_hid(raw: &[u8]) -> Vec<App> {
    let mut apps: Vec<App> = Vec::new();
    let (mut page, mut min, mut max, mut size, mut count) = (0u32, 0i64, 0i64, 0u32, 0u32);
    let (mut phys_min, mut phys_max, mut unit, mut exponent, mut report_id) = (0i64, 0i64, 0u32, 0i32, 0u32);
    let mut usages: Vec<u32> = Vec::new();
    let (mut umin, mut umax) = (None::<u32>, None::<u32>);
    let mut depth = 0i32;
    let mut stack: Vec<(u32, i64, i64, u32, u32, i64, i64, u32, i32, u32)> = Vec::new();

    let mut i = 0;
    while i < raw.len() {
        let prefix = raw[i];
        if prefix == 0xfe {
            i += 2 + raw.get(i + 1).copied().unwrap_or(0) as usize;
            continue;
        }
        let len = match prefix & 3 {
            3 => 4,
            n => n as usize,
        };
        let data = &raw[(i + 1).min(raw.len())..(i + 1 + len).min(raw.len())];
        let value = data.iter().enumerate().fold(0u32, |a, (k, b)| a | (*b as u32) << (8 * k));
        let (kind, tag) = ((prefix >> 2) & 3, prefix >> 4);
        i += 1 + len;

        match (kind, tag) {
            // global
            (1, 0) => page = value,
            (1, 1) => min = signed(value, len),
            (1, 2) => max = if len == 0 { 0 } else { signed(value, len) },
            (1, 3) => phys_min = signed(value, len),
            (1, 4) => phys_max = signed(value, len),
            (1, 5) => exponent = ((value & 0xf) as i32 ^ 8) - 8,
            (1, 6) => unit = value,
            (1, 7) => size = value,
            (1, 8) => report_id = value,
            (1, 9) => count = value,
            (1, 0xa) => stack.push((page, min, max, size, count, phys_min, phys_max, unit, exponent, report_id)),
            (1, 0xb) => {
                if let Some(s) = stack.pop() {
                    (page, min, max, size, count, phys_min, phys_max, unit, exponent, report_id) = s;
                }
            }
            // local
            (2, 0) => usages.push(if len == 4 { value } else { (page << 16) | value }),
            (2, 1) => umin = Some(value),
            (2, 2) => umax = Some(value),
            // main
            (0, 0xa) => {
                if depth == 0 && value == 1 {
                    let u = usages.first().copied().unwrap_or(0);
                    apps.push(App { page: u >> 16, usage: u & 0xffff, fields: Vec::new() });
                }
                depth += 1;
                usages.clear();
                (umin, umax) = (None, None);
            }
            (0, 0xc) => depth -= 1,
            (0, 8 | 9 | 0xb) => {
                let mut list = usages.clone();
                if let (Some(a), Some(b)) = (umin, umax) {
                    list.extend((a..=b.min(a + 255)).map(|u| (page << 16) | u));
                }
                if let Some(app) = apps.last_mut() {
                    app.fields.push(Field {
                        page,
                        usages: list,
                        size,
                        count,
                        min,
                        max,
                        phys_min,
                        phys_max,
                        unit,
                        exponent,
                        flags: value,
                        kind: match tag {
                            8 => "input",
                            9 => "output",
                            _ => "feature",
                        },
                        report_id,
                    });
                }
                usages.clear();
                (umin, umax) = (None, None);
            }
            _ => {}
        }
    }
    apps
}

/// Turns the parsed descriptor into short "label → value" facts.
fn decode_hid(raw: &[u8]) -> Vec<(String, String)> {
    let apps = parse_hid(raw);
    let mut rows = Vec::new();
    let multiple = apps.len() > 1;

    for app in &apps {
        let app_name = usage_name(app.page, app.usage);
        let mut push = |what: &str, value: String| {
            let label = if multiple { format!("{app_name} · {what}") } else { what.to_owned() };
            rows.push((label, value));
        };
        push("Function", app_name.clone());

        let ids: std::collections::BTreeSet<u32> = app.fields.iter().map(|f| f.report_id).collect();
        if ids.iter().any(|i| *i != 0) {
            push("Report IDs", ids.iter().map(|i| i.to_string()).collect::<Vec<_>>().join(", "));
        }

        let mut buttons = 0;
        let mut vendor_bits = 0;
        let mut leds: Vec<&str> = Vec::new();
        for f in &app.fields {
            let variable = f.flags & 2 != 0;
            let relative = f.flags & 4 != 0;
            let constant = f.flags & 1 != 0;
            if constant {
                continue;
            }
            match f.page {
                0x09 if f.kind == "input" => buttons += f.usages.len().max(f.count as usize),
                0x08 if f.kind == "output" => {
                    leds.extend(f.usages.iter().filter_map(|u| led_name((*u & 0xffff) as usize - 1)));
                }
                0x07 if variable && f.usages.len() == 8 => push("Modifier keys", "8 (Ctrl, Shift, Alt, Meta × left/right)".into()),
                0x07 if !variable && f.kind == "input" => push(
                    "Key reports",
                    format!("up to {} keys pressed at once, key codes 0–{}", f.count, f.max),
                ),
                p if p > 0x100 => vendor_bits += f.size * f.count,
                0x0c if !variable && f.kind == "input" => {
                    push("Media / consumer keys", format!("up to {} at once (key codes 0–{})", f.count, f.max));
                }
                _ if variable => {
                    for u in &f.usages {
                        let name = usage_name(f.page, u & 0xffff);
                        let kind = if relative { "relative" } else { "absolute" };
                        let mut text = format!("{}-bit {kind}, {}…{}", f.size, f.min, f.max);
                        if f.kind == "feature" {
                            text = format!("settable, {}…{}", f.min, f.max);
                        }
                        // counts per inch, when the device declares physical units
                        if f.phys_max != f.phys_min && matches!(f.unit & 0xff, 0x11 | 0x13) {
                            let span = (f.phys_max - f.phys_min) as f64 * 10f64.powi(f.exponent);
                            let inches = if f.unit & 0xff == 0x13 { span } else { span / 2.54 };
                            if inches > 0.0 {
                                text += &format!(" · ~{:.0} counts/inch", (f.max - f.min) as f64 / inches);
                            }
                        }
                        push(&name, text);
                    }
                }
                _ => {}
            }
        }
        if buttons > 0 {
            push("Buttons", buttons.to_string());
        }
        if !leds.is_empty() {
            push("Indicator LEDs", leds.join(", "));
        }
        if vendor_bits > 0 {
            push("Vendor-defined data", format!("{vendor_bits} bits per report (proprietary protocol)"));
        }
    }
    if apps.is_empty() {
        rows.push(("Contents".into(), "Could not be decoded".into()));
    }
    rows
}

// -------------------------------------------------------------- PCI helper

#[cfg(target_os = "linux")]
fn pci_rows(d: &mut Details, section: &str, device_dir: &Path) {
    let hex = |f: &str| read(device_dir.join(f)).map(|v| v.trim_start_matches("0x").to_owned());
    let (vendor, device) = (hex("vendor"), hex("device"));
    if let (Some(v), Some(dv)) = (&vendor, &device) {
        let (vname, dname) = lookup_ids(PCI_IDS, v, Some(dv));
        d.add_opt(section, "Vendor", vname);
        d.add_opt(section, "Model", dname);
        d.add(section, "PCI ID", format!("{v}:{dv}"));
    }
    if let Some(slot) = std::fs::canonicalize(device_dir).ok().and_then(|p| p.file_name().map(|n| n.to_string_lossy().into_owned())) {
        d.add(section, "PCI address", slot);
    }
    let driver = std::fs::read_link(device_dir.join("driver"))
        .ok()
        .and_then(|p| p.file_name().map(|f| f.to_string_lossy().into_owned()));
    if let Some(driver) = driver {
        driver_rows(d, &format!("{section} · driver"), &driver);
    }
}

// --------------------------------------------------------------- audio jack

#[cfg(target_os = "linux")]
fn jack(d: &mut Details, rest: &str) {
    let Some(card) = rest.split('-').next().and_then(|c| c.parse::<u32>().ok()) else {
        return;
    };
    let section = "Sound card";
    d.add_opt(section, "Card ID", read(format!("/proc/asound/card{card}/id")));
    if let Ok(cards) = std::fs::read_to_string("/proc/asound/cards") {
        let mut lines = cards.lines();
        while let Some(line) = lines.next() {
            if line.trim_start().starts_with(&format!("{card} [")) {
                d.add_opt(section, "Description", lines.next().map(|l| l.trim().to_owned()));
            }
        }
    }

    if let Ok(codec) = std::fs::read_to_string(format!("/proc/asound/card{card}/codec#0")) {
        for line in codec.lines().take(8) {
            if let Some((k, v)) = line.split_once(':') {
                if matches!(k, "Codec" | "Address" | "Vendor Id" | "Subsystem Id" | "Revision Id") {
                    d.add("Audio codec chip", k, v.trim());
                }
            }
        }
    }

    if let Ok(pcm) = std::fs::read_to_string("/proc/asound/pcm") {
        for line in pcm.lines().filter(|l| l.starts_with(&format!("{card:02}-"))) {
            let parts: Vec<&str> = line.split(" : ").collect();
            if parts.len() >= 3 {
                d.add("Streams", parts[0].trim(), format!("{} · {}", parts[1].trim(), parts[2..].join(", ")));
            }
        }
    }

    if let Ok(out) = Command::new("amixer").args(["-c", &card.to_string(), "scontrols"]).output() {
        let names: Vec<String> = String::from_utf8_lossy(&out.stdout)
            .lines()
            .filter_map(|l| l.split('\'').nth(1).map(str::to_owned))
            .filter(|n| n.contains("Mic") || n.contains("Capture") || n.contains("Headphone"))
            .collect();
        if !names.is_empty() {
            d.add("Streams", "Related mixer controls", names.join(", "));
        }
    }

    pci_rows(d, "Audio controller", &PathBuf::from(format!("/sys/class/sound/card{card}/device")));
}

// ------------------------------------------------------------------ display

#[cfg(target_os = "linux")]
fn display(d: &mut Details, connector: &str) {
    let Some(dir) = std::fs::read_dir("/sys/class/drm").ok().and_then(|it| {
        it.flatten().map(|e| e.path()).find(|p| {
            p.file_name()
                .map(|n| n.to_string_lossy().split_once('-').map(|(_, c)| c == connector).unwrap_or(false))
                .unwrap_or(false)
        })
    }) else {
        d.add("Connection", "Status", "Connector not found");
        return;
    };

    d.add("Connection", "Connector", connector);
    d.add_opt("Connection", "Status", read(dir.join("status")));
    d.add_opt("Connection", "Enabled", read(dir.join("enabled")));
    d.add_opt("Connection", "Power state (DPMS)", read(dir.join("dpms")));

    if let Ok(modes) = std::fs::read_to_string(dir.join("modes")) {
        let mut seen = Vec::new();
        for m in modes.lines().map(str::trim).filter(|m| !m.is_empty()) {
            if !seen.contains(&m) {
                seen.push(m);
            }
        }
        if !seen.is_empty() {
            d.add("Supported modes", "Preferred", seen[0]);
            d.add(
                "Supported modes",
                format!("All ({})", seen.len()),
                seen.iter().take(24).copied().collect::<Vec<_>>().join(", "),
            );
        }
    }

    if let Ok(raw) = std::fs::read(dir.join("edid")) {
        edid_extra(d, &raw);
    }

    // The graphics adapter driving this connector.
    // connector/device → the card; card/device → the PCI adapter itself
    let card = dir.join("device");
    let pci = if card.join("vendor").exists() { card } else { card.join("device") };
    pci_rows(d, "Graphics adapter", &pci);
}

fn edid_extra(d: &mut Details, raw: &[u8]) {
    if raw.len() < 128 {
        return;
    }
    let s = "Panel (from EDID)";
    d.add(s, "EDID version", format!("{}.{}", raw[18], raw[19]));

    let id = u16::from_be_bytes([raw[8], raw[9]]);
    let code: String = [id >> 10, id >> 5, id].iter().map(|v| (b'A' + (*v as u8 & 0x1f) - 1) as char).collect();
    d.add_opt(s, "Registered manufacturer", pnp_vendor(&code));

    let input = raw[20];
    if input & 0x80 != 0 {
        let depth = match (input >> 4) & 7 {
            1 => "6-bit",
            2 => "8-bit",
            3 => "10-bit",
            4 => "12-bit",
            5 => "14-bit",
            6 => "16-bit",
            _ => "unspecified depth",
        };
        let interface = match input & 0xf {
            1 => "DVI",
            2 => "HDMI-a",
            3 => "HDMI-b",
            4 => "MDDI",
            5 => "DisplayPort",
            _ => "digital",
        };
        d.add(s, "Signal", format!("Digital · {interface} · {depth} colour"));
    } else {
        d.add(s, "Signal", "Analog");
    }
    if raw[23] != 0xff {
        d.add(s, "Gamma", format!("{:.2}", (raw[23] as f64 + 100.0) / 100.0));
    }

    for block in raw[54..126].chunks(18) {
        let clock = u16::from_le_bytes([block[0], block[1]]) as f64 * 10_000.0;
        if clock > 0.0 {
            let h = block[2] as u32 | ((block[4] as u32 & 0xf0) << 4);
            let hb = block[3] as u32 | ((block[4] as u32 & 0x0f) << 8);
            let v = block[5] as u32 | ((block[7] as u32 & 0xf0) << 4);
            let vb = block[6] as u32 | ((block[7] as u32 & 0x0f) << 8);
            let refresh = clock / ((h + hb) as f64 * (v + vb) as f64);
            d.add(s, "Preferred mode", format!("{h} × {v} @ {refresh:.2} Hz · {:.1} MHz pixel clock", clock / 1e6));
        } else if block[3] == 0xfd {
            d.add(
                s,
                "Refresh range",
                format!("{}–{} Hz vertical · {}–{} kHz horizontal", block[5], block[6], block[7], block[8]),
            );
            if block[9] != 0 {
                d.add(s, "Max pixel clock", format!("{} MHz", block[9] as u32 * 10));
            }
        }
    }
    d.add(s, "EDID blocks", format!("{} (extension blocks: {})", raw.len() / 128, raw[126]));
}

// ----------------------------------------------------------------- computer

#[cfg(target_os = "linux")]
fn computer(d: &mut Details) {
    // graphics adapters
    let mut seen = Vec::new();
    for e in std::fs::read_dir("/sys/class/drm").into_iter().flatten().flatten() {
        let name = e.file_name().to_string_lossy().into_owned();
        if name.starts_with("card") && !name.contains('-') {
            let dev = e.path().join("device");
            if let Ok(real) = std::fs::canonicalize(&dev) {
                if !seen.contains(&real) {
                    seen.push(real);
                    pci_rows(d, &format!("Graphics adapter · {name}"), &dev);
                }
            }
        }
    }

    // processor details beyond the basics
    let cpu = Path::new("/sys/devices/system/cpu/cpu0");
    let khz = |f: &str| read(cpu.join("cpufreq").join(f)).and_then(|v| v.parse::<f64>().ok());
    if let (Some(lo), Some(hi)) = (khz("cpuinfo_min_freq"), khz("cpuinfo_max_freq")) {
        d.add("Processor clock", "Range", format!("{:.1} – {:.1} GHz", lo / 1e6, hi / 1e6));
    }
    d.add_opt("Processor clock", "Scaling driver", read(cpu.join("cpufreq/scaling_driver")));
    d.add_opt("Processor clock", "Governor", read(cpu.join("cpufreq/scaling_governor")));
    for i in 0..6 {
        let c = cpu.join(format!("cache/index{i}"));
        if let (Some(level), Some(kind), Some(size)) = (read(c.join("level")), read(c.join("type")), read(c.join("size"))) {
            d.add("Processor cache", format!("L{level} {}", kind.to_lowercase()), size);
        }
    }

    // storage volumes
    for disk in sysinfo::Disks::new_with_refreshed_list().iter().filter(|x| x.total_space() > 0) {
        d.add(
            "Storage volumes",
            disk.mount_point().to_string_lossy().into_owned(),
            format!(
                "{} · {:?} · {} total, {} free",
                disk.file_system().to_string_lossy(),
                disk.kind(),
                format_bytes(disk.total_space()),
                format_bytes(disk.available_space())
            ),
        );
    }

    // network adapters
    for (name, data) in sysinfo::Networks::new_with_refreshed_list().iter().filter(|(n, _)| n.as_str() != "lo") {
        let ips: Vec<String> = data.ip_networks().iter().map(|ip| ip.addr.to_string()).collect();
        d.add(
            "Network adapters",
            name.clone(),
            format!("{} · {}", data.mac_address(), if ips.is_empty() { "no address".into() } else { ips.join(", ") }),
        );
    }
}
