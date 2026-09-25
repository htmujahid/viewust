use crate::detail::{self, Detail, DisplayFacts};
use nusb::{DeviceInfo, MaybeFuture};
use serde::Serialize;
use sysinfo::System;
use tauri::AppHandle;

#[derive(Serialize)]
pub struct Peripheral {
    id: String,
    /// Id of the receiver this device talks to wirelessly, if any.
    via: Option<String>,
    /// True when the link to its parent is radio rather than a cable.
    wireless: bool,
    name: String,
    manufacturer: Option<String>,
    /// Drives which illustration the UI draws.
    kind: &'static str,
    connection: String,
    vendor_id: String,
    product_id: String,
    serial_number: Option<String>,
    details: Vec<Detail>,
}

#[derive(Serialize)]
pub struct Display {
    name: String,
    /// Connector it is plugged into ("DP-3"), when known.
    connector: Option<String>,
    /// Real panel width, used to draw the monitor at its true relative size.
    width_cm: Option<u32>,
    width: u32,
    height: u32,
    scale_factor: f64,
    primary: bool,
    details: Vec<Detail>,
}

#[derive(Serialize)]
pub struct HardwareInfo {
    computer_name: String,
    computer_details: Vec<Detail>,
    connection: Option<crate::connection::Connection>,
    peripherals: Vec<Peripheral>,
    displays: Vec<Display>,
}

const HID: u8 = 0x03;

/// Every function a USB device provides. A single receiver can expose a
/// keyboard and a mouse at once, so this returns more than one kind.
fn classify(device: &DeviceInfo) -> Vec<&'static str> {
    let name = format!(
        "{} {}",
        device.manufacturer_string().unwrap_or_default(),
        device.product_string().unwrap_or_default()
    )
    .to_lowercase();
    let has = |words: &[&str]| words.iter().any(|w| name.contains(w));

    let mut classes: Vec<(u8, u8)> = device
        .interfaces()
        .map(|i| (i.class(), i.protocol()))
        .collect();
    classes.push((device.class(), 0));
    let class = |c: u8| classes.iter().any(|(class, _)| *class == c);
    let hid_protocol = |p: u8| classes.iter().any(|(c, proto)| *c == HID && *proto == p);

    let mut kinds: Vec<&'static str> = Vec::new();
    let mut add = |kind: &'static str, found: bool| {
        if found && !kinds.contains(&kind) {
            kinds.push(kind);
        }
    };

    let webcam = has(&["webcam", "camera", "c920", "c922", "brio"]) || class(0x0e);
    add("webcam", webcam);
    // A "mic" in the name is matched as a whole word: "Microsoft" contains it.
    let padded = format!(" {name} ");
    let microphone = has(&["microphone", "yeti", "snowball", "podcast", "condenser"])
        || padded.contains(" mic ");
    add("microphone", microphone);
    // Webcams carry a microphone interface; it isn't a separate device.
    add(
        "audio",
        !webcam
            && !microphone
            && (has(&["headset", "headphone", "earbud", "airpods", "speaker", "dac"])
                || class(0x01)),
    );
    add(
        "gamepad",
        has(&["gamepad", "controller", "joystick", "xbox", "dualshock", "dualsense"]),
    );
    add("keyboard", has(&["keyboard"]) || hid_protocol(1));
    add(
        "mouse",
        has(&["mouse", "trackball", "touchpad", "trackpad"]) || hid_protocol(2),
    );
    add("printer", class(0x07) || has(&["printer"]));
    add(
        "storage",
        class(0x08) || has(&["flash", "storage", "ssd", "disk", "card reader"]),
    );
    add("securitykey", class(0x0b) || has(&["smart card", "smartcard", "yubikey"]));
    add("phone", class(0x06) || has(&["iphone", "ipad", "android", "pixel", "galaxy", "phone"]));

    // Gaming keyboards often expose a spare mouse interface for macros, and
    // mice a keyboard one. When the product name says which it is, trust it.
    let says_keyboard = has(&["keyboard"]);
    let says_mouse = has(&["mouse", "trackball"]);
    if says_keyboard && !says_mouse {
        kinds.retain(|k| *k != "mouse");
    } else if says_mouse && !says_keyboard {
        kinds.retain(|k| *k != "keyboard");
    }

    if kinds.is_empty() {
        let fallback = if class(0xe0) || has(&["bluetooth", "wireless", "wi-fi", "wlan", "dongle", "receiver"]) {
            "wireless"
        } else if class(0x09) {
            "hub"
        } else {
            "generic"
        };
        kinds.push(fallback);
    }
    kinds
}

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

/// Linux marks USB devices soldered into the machine (internal webcam,
/// Bluetooth chip, fingerprint reader) as `fixed`. Those aren't plugged in.
#[cfg(target_os = "linux")]
fn is_internal(device: &DeviceInfo) -> bool {
    std::fs::read_to_string(device.sysfs_path().join("removable"))
        .is_ok_and(|v| v.trim() == "fixed")
}

#[cfg(not(target_os = "linux"))]
fn is_internal(_: &DeviceInfo) -> bool {
    false
}

/// A dongle that carries wireless input devices (keyboard, mouse, ...) rather
/// than being one of them.
fn is_receiver(device: &DeviceInfo, kinds: &[&str]) -> bool {
    let name = format!(
        "{} {}",
        device.manufacturer_string().unwrap_or_default(),
        device.product_string().unwrap_or_default()
    )
    .to_lowercase();
    let carries_input = kinds
        .iter()
        .all(|k| matches!(*k, "keyboard" | "mouse" | "gamepad" | "audio" | "microphone"));
    let both_inputs = kinds.contains(&"keyboard") && kinds.contains(&"mouse");
    let says_receiver = ["receiver", "unifying", "dongle", "2.4g", "wireless", "bluetooth"]
        .iter()
        .any(|w| name.contains(w));
    carries_input && (both_inputs || says_receiver)
}

/// Analog 3.5 mm jacks with something plugged in. The sound card reports each
/// jack's state, so a microphone on the front panel is detectable even though
/// it never appears on the USB bus. Linux only; needs `amixer` (alsa-utils).
#[cfg(target_os = "linux")]
fn audio_jacks() -> Vec<Peripheral> {
    use std::process::Command;

    let Ok(cards) = std::fs::read_to_string("/proc/asound/cards") else {
        return Vec::new();
    };

    let mut out = Vec::new();
    for line in cards.lines() {
        // " 0 [PCH            ]: HDA-Intel - HDA Intel PCH"
        let Some((index, rest)) = line.trim_start().split_once(' ') else {
            continue;
        };
        let Ok(index) = index.parse::<u32>() else {
            continue;
        };
        // USB sound cards are already listed from the USB bus.
        if std::path::Path::new(&format!("/proc/asound/card{index}/usbid")).exists() {
            continue;
        }
        let card_name = rest.rsplit_once(" - ").map(|(_, n)| n.trim().to_owned());

        let Ok(output) = Command::new("amixer")
            .args(["-c", &index.to_string(), "contents"])
            .output()
        else {
            continue;
        };
        let text = String::from_utf8_lossy(&output.stdout);

        // Blocks look like:  numid=32,iface=CARD,name='Front Mic Jack'
        //                      ; type=BOOLEAN ...
        //                      : values=on
        let mut current: Option<String> = None;
        for line in text.lines() {
            if line.starts_with("numid=") {
                current = line
                    .contains("iface=CARD")
                    .then(|| line.split("name='").nth(1))
                    .flatten()
                    .and_then(|n| n.strip_suffix('\''))
                    .filter(|n| n.ends_with(" Jack"))
                    .map(str::to_owned);
            } else if line.trim() == ": values=on" {
                if let Some(jack) = current.take() {
                    out.extend(jack_device(index, &jack, card_name.clone()));
                }
            }
        }
    }
    out
}

#[cfg(not(target_os = "linux"))]
fn audio_jacks() -> Vec<Peripheral> {
    Vec::new()
}

#[cfg(target_os = "linux")]
fn jack_device(card: u32, jack: &str, card_name: Option<String>) -> Option<Peripheral> {
    let label = jack.trim_end_matches(" Jack");
    let lower = label.to_lowercase();
    let (kind, name) = if lower.contains("mic") {
        ("microphone", format!("{label} microphone").replace(" Mic microphone", " microphone"))
    } else if lower.contains("headphone") {
        ("audio", format!("{label}s"))
    } else if lower.contains("line out") || lower.contains("speaker") {
        ("audio", format!("{label} speakers"))
    } else {
        return None;
    };
    let details = jack_details(card, jack, card_name.as_deref());
    Some(Peripheral {
        id: format!("jack-{card}-{}", lower.replace(' ', "-")),
        via: None,
        wireless: false,
        name,
        manufacturer: card_name,
        kind,
        connection: "3.5 mm jack".into(),
        vendor_id: String::new(),
        product_id: String::new(),
        serial_number: None,
        details,
    })
}

#[cfg(target_os = "linux")]
fn jack_details(card: u32, jack: &str, card_name: Option<&str>) -> Vec<Detail> {
    let mut d = detail::Details::new();
    d.add("Connection", "Type", "3.5 mm analog jack");
    d.add("Connection", "Jack", jack.trim_end_matches(" Jack"));
    d.add("Connection", "State", "Something is plugged in");
    d.add_opt("Sound card", "Card", card_name);
    d.add("Sound card", "Card number", card.to_string());
    let codec = std::fs::read_to_string(format!("/proc/asound/card{card}/codec#0")).ok().and_then(|t| {
        t.lines()
            .find_map(|l| l.strip_prefix("Codec:").map(|c| c.trim().to_owned()))
    });
    d.add_opt("Sound card", "Audio codec", codec);
    d.finish()
}

/// Root hubs are the computer's own USB controllers, not something plugged in.
fn is_root_hub(device: &DeviceInfo) -> bool {
    device.vendor_id() == 0x1d6b && device.class() == 0x09
}

fn peripherals() -> Vec<Peripheral> {
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
            details: detail::usb_details(&d),
        };

        if receiver {
            // The dongle is plugged in by cable; what it carries is wireless.
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
    out.extend(audio_jacks());
    out.sort_by(|a, b| a.kind.cmp(b.kind).then_with(|| a.name.cmp(&b.name)));
    out
}

/// The laptop's own panel is part of the computer, not an attached device.
fn is_built_in(name: &str) -> bool {
    let n = name.to_lowercase();
    n.starts_with("edp") || n.starts_with("lvds") || n.starts_with("dsi") || n.contains("built-in")
}

fn displays(app: &AppHandle) -> Vec<Display> {
    let primary = app.primary_monitor().ok().flatten();
    let mut edids = detail::connected_edids();

    app.available_monitors()
        .unwrap_or_default()
        .into_iter()
        .filter(|m| !m.name().is_some_and(|n| is_built_in(n)))
        .map(|m| {
            let name = m.name().cloned().unwrap_or_else(|| "Display".into());
            let is_primary = primary
                .as_ref()
                .is_some_and(|p| p.position() == m.position() && p.size() == m.size());

            // Toolkits name a monitor by connector ("DP-3") or by model;
            // match either, otherwise hand out the next unclaimed one.
            let found = edids
                .iter()
                .position(|(connector, e)| *connector == name || e.model() == Some(name.as_str()))
                .or_else(|| (!edids.is_empty()).then_some(0));
            let (connector, edid) = match found {
                Some(i) => {
                    let (c, e) = edids.remove(i);
                    (Some(c), Some(e))
                }
                None => (None, None),
            };

            let details = detail::display_details(&DisplayFacts {
                width: m.size().width,
                height: m.size().height,
                x: m.position().x,
                y: m.position().y,
                scale_factor: m.scale_factor(),
                primary: is_primary,
                connector: connector.as_deref(),
                edid: edid.as_ref(),
            });
            Display {
                connector: connector.clone(),
                width_cm: edid.as_ref().and_then(|e| e.width_cm()),
                name: edid.as_ref().and_then(|e| e.model()).map(str::to_owned).unwrap_or(name),
                width: m.size().width,
                height: m.size().height,
                scale_factor: m.scale_factor(),
                primary: is_primary,
                details,
            }
        })
        .collect()
}

#[tauri::command]
pub fn hardware_info(app: AppHandle) -> HardwareInfo {
    HardwareInfo {
        computer_name: System::host_name().unwrap_or_else(|| "This computer".into()),
        computer_details: detail::computer_details(),
        connection: crate::connection::connection(),
        peripherals: peripherals(),
        displays: displays(&app),
    }
}
