//! The technical report for a USB device.

use super::{descriptors::*, driver::driver_rows, hid::hid_section};
use crate::common::ids::*;
use crate::common::sysfs::*;
use crate::common::Details;
use std::path::{Path, PathBuf};
use std::process::Command;

pub(crate) struct Iface {
    pub(crate) path: PathBuf,
    pub(crate) number: u8,
    pub(crate) class: u8,
    pub(crate) protocol: u8,
}

pub(crate) fn interfaces(sys: &Path) -> Vec<Iface> {
    let base = sys
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .into_owned();
    let mut out: Vec<Iface> = std::fs::read_dir(sys)
        .into_iter()
        .flatten()
        .flatten()
        .filter(|e| {
            e.file_name()
                .to_string_lossy()
                .starts_with(&format!("{base}:"))
        })
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
pub(crate) fn report(d: &mut Details, id: &str) {
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

    let speed_mbps = read(sys.join("speed"))
        .and_then(|s| s.parse::<f64>().ok())
        .unwrap_or(12.0);
    let desc = std::fs::read(sys.join("descriptors"))
        .map(|r| parse_descriptors(&r))
        .unwrap_or_default();

    if let Some(a) = desc.config_attributes {
        let mut parts = vec![if a & 0x40 != 0 {
            "Self-powered"
        } else {
            "Bus-powered"
        }];
        if a & 0x20 != 0 {
            parts.push("remote wake-up (can wake the computer)");
        }
        d.add("Power", "Configuration", parts.join(" · "));
    }

    let ifaces: Vec<Iface> = interfaces(&sys)
        .into_iter()
        .filter(|i| wanted(kind, i))
        .collect();

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
            numbers
                .iter()
                .map(u8::to_string)
                .collect::<Vec<_>>()
                .join(", "),
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
                    d.add(
                        format!("Interface {n} · video"),
                        format!("/dev/{node}"),
                        label,
                    );
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
    let eps: Vec<&Endpoint> = desc
        .endpoints
        .iter()
        .filter(|e| shown.contains(&e.interface))
        .collect();
    for e in eps.iter().take(40) {
        let alt = if e.alt > 0 {
            format!(" alt {}", e.alt)
        } else {
            String::new()
        };
        d.add(
            "Endpoints",
            format!("Interface {}{alt} · 0x{:02x}", e.interface, e.address),
            endpoint_text(e, speed_mbps),
        );
    }
    if eps.len() > 40 {
        d.add(
            "Endpoints",
            "…",
            format!("{} more endpoints not shown", eps.len() - 40),
        );
    }
}

#[cfg(target_os = "linux")]
fn audio_streams(d: &mut Details, iface: u8, card: u32) {
    let section = format!("Interface {iface} · audio");
    d.add_opt(
        &section,
        "Card",
        read(format!("/proc/asound/card{card}/id")),
    );
    for n in 0..4 {
        let Ok(text) = std::fs::read_to_string(format!("/proc/asound/card{card}/stream{n}")) else {
            break;
        };
        let mut direction = "";
        let (mut formats, mut channels, mut rates): (Vec<String>, Vec<u32>, Vec<u32>) =
            Default::default();
        let flush = |d: &mut Details,
                     direction: &str,
                     f: &mut Vec<String>,
                     c: &mut Vec<u32>,
                     r: &mut Vec<u32>| {
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
                [f.join(", "), chans, hz]
                    .into_iter()
                    .filter(|s| !s.is_empty())
                    .collect::<Vec<_>>()
                    .join(" · "),
            );
            f.clear();
            c.clear();
            r.clear();
        };
        for line in text.lines().map(str::trim) {
            if line == "Capture:" || line == "Playback:" {
                flush(d, direction, &mut formats, &mut channels, &mut rates);
                direction = if line == "Capture:" {
                    "Recording"
                } else {
                    "Playback"
                };
            } else if let Some(v) = line.strip_prefix("Format:") {
                formats.push(v.trim().to_owned());
            } else if let Some(v) = line.strip_prefix("Channels:") {
                channels.extend(
                    v.split(|c: char| !c.is_ascii_digit())
                        .filter_map(|x| x.parse::<u32>().ok()),
                );
            } else if let Some(v) = line.strip_prefix("Rates:") {
                rates.extend(
                    v.split(|c: char| !c.is_ascii_digit())
                        .filter_map(|x| x.parse::<u32>().ok()),
                );
            }
        }
        flush(d, direction, &mut formats, &mut channels, &mut rates);
    }
    if let Ok(out) = Command::new("amixer")
        .args(["-c", &card.to_string(), "scontrols"])
        .output()
    {
        let names: Vec<String> = String::from_utf8_lossy(&out.stdout)
            .lines()
            .filter_map(|l| l.split('\'').nth(1).map(str::to_owned))
            .collect();
        if !names.is_empty() {
            d.add(&section, "Mixer controls", names.join(", "));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn iface(class: u8, protocol: u8) -> Iface {
        Iface {
            path: PathBuf::new(),
            number: 0,
            class,
            protocol,
        }
    }

    #[test]
    fn a_receivers_keyboard_and_mouse_get_their_own_interfaces() {
        let (kbd, mouse) = (iface(3, 1), iface(3, 2));
        assert!(wanted(Some("keyboard"), &kbd) && !wanted(Some("keyboard"), &mouse));
        assert!(wanted(Some("mouse"), &mouse) && !wanted(Some("mouse"), &kbd));
        // opening the receiver itself (no function) shows everything
        assert!(wanted(None, &kbd) && wanted(None, &mouse));
    }
}
