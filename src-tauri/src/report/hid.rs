//! HID report descriptors and the input devices the kernel builds from them.

use super::input::*;
use crate::common::sysfs::*;
use crate::common::Details;
use std::path::Path;

#[cfg(target_os = "linux")]
pub(crate) fn hid_section(
    d: &mut Details,
    iface: u8,
    hid_dir: &Path,
    hid: Option<(u16, u8, u16)>,
    dev: &nusb::DeviceInfo,
) {
    let name = |what: &str| format!("Interface {iface} · {what}");
    let product = dev.product_string().unwrap_or_default();
    let maker = dev.manufacturer_string().unwrap_or_default();

    // The kernel's input devices created for this interface.
    for e in std::fs::read_dir(hid_dir.join("input"))
        .into_iter()
        .flatten()
        .flatten()
    {
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
        d.add(
            &section,
            "Device nodes",
            handlers
                .iter()
                .map(|h| format!("/dev/input/{h}"))
                .collect::<Vec<_>>()
                .join(", "),
        );

        let cap = |f: &str| {
            read(p.join("capabilities").join(f))
                .map(|t| bitmap(&t))
                .unwrap_or_default()
        };
        let ev = cap("ev");
        d.add(&section, "Event types", named(&ev, ev_name).join(", "));

        let key = cap("key");
        let buttons = named(&key, button_name);
        if !buttons.is_empty() {
            d.add(
                &section,
                format!("Buttons ({})", buttons.len()),
                buttons.join(", "),
            );
        }
        let keys = key.iter().filter(|k| **k < 0x100).count();
        if keys > 0 {
            d.add(&section, "Keyboard keys", format!("{keys} key codes"));
        }
        let extra = key
            .iter()
            .filter(|k| **k >= 0x160 && button_name(**k).is_none())
            .count();
        if extra > 0 {
            d.add(&section, "Extra / media key codes", extra.to_string());
        }
        d.add(
            &section,
            "Movement axes",
            named(&cap("rel"), rel_name).join(", "),
        );
        d.add(
            &section,
            "Absolute axes",
            named(&cap("abs"), abs_name).join(", "),
        );
        d.add(
            &section,
            "Indicator LEDs",
            named(&cap("led"), led_name).join(", "),
        );
    }

    // The raw HID report descriptor, decoded.
    let Ok(raw) = std::fs::read(hid_dir.join("report_descriptor")) else {
        return;
    };
    let section = name("HID report descriptor");
    if let Some((version, country, len)) = hid {
        d.add(
            &section,
            "HID version",
            format!("{:x}.{:02x}", version >> 8, version & 0xff),
        );
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
        .map(|c| {
            c.iter()
                .map(|b| format!("{b:02x}"))
                .collect::<Vec<_>>()
                .join(" ")
        })
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

/// The "global" items of a report descriptor. They persist from one data item
/// to the next, and can be saved and restored with push/pop.
#[derive(Clone, Copy, Default)]
struct Globals {
    page: u32,
    min: i64,
    max: i64,
    phys_min: i64,
    phys_max: i64,
    exponent: i32,
    unit: u32,
    size: u32,
    count: u32,
    report_id: u32,
}

fn parse_hid(raw: &[u8]) -> Vec<App> {
    let mut apps: Vec<App> = Vec::new();
    let mut g = Globals::default();
    let mut saved: Vec<Globals> = Vec::new();
    let mut usages: Vec<u32> = Vec::new();
    let (mut umin, mut umax) = (None::<u32>, None::<u32>);
    let mut depth = 0i32;

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
        let value = data
            .iter()
            .enumerate()
            .fold(0u32, |a, (k, b)| a | (*b as u32) << (8 * k));
        let (kind, tag) = ((prefix >> 2) & 3, prefix >> 4);
        i += 1 + len;

        match (kind, tag) {
            // global
            (1, 0) => g.page = value,
            (1, 1) => g.min = signed(value, len),
            (1, 2) => g.max = if len == 0 { 0 } else { signed(value, len) },
            (1, 3) => g.phys_min = signed(value, len),
            (1, 4) => g.phys_max = signed(value, len),
            (1, 5) => g.exponent = ((value & 0xf) as i32 ^ 8) - 8,
            (1, 6) => g.unit = value,
            (1, 7) => g.size = value,
            (1, 8) => g.report_id = value,
            (1, 9) => g.count = value,
            (1, 0xa) => saved.push(g),
            (1, 0xb) => {
                if let Some(previous) = saved.pop() {
                    g = previous;
                }
            }
            // local
            (2, 0) => usages.push(if len == 4 {
                value
            } else {
                (g.page << 16) | value
            }),
            (2, 1) => umin = Some(value),
            (2, 2) => umax = Some(value),
            // main
            (0, 0xa) => {
                if depth == 0 && value == 1 {
                    let u = usages.first().copied().unwrap_or(0);
                    apps.push(App {
                        page: u >> 16,
                        usage: u & 0xffff,
                        fields: Vec::new(),
                    });
                }
                depth += 1;
                usages.clear();
                (umin, umax) = (None, None);
            }
            (0, 0xc) => depth -= 1,
            (0, 8 | 9 | 0xb) => {
                let mut list = usages.clone();
                if let (Some(a), Some(b)) = (umin, umax) {
                    list.extend((a..=b.min(a + 255)).map(|u| (g.page << 16) | u));
                }
                if let Some(app) = apps.last_mut() {
                    app.fields.push(Field {
                        page: g.page,
                        usages: list,
                        size: g.size,
                        count: g.count,
                        min: g.min,
                        max: g.max,
                        phys_min: g.phys_min,
                        phys_max: g.phys_max,
                        unit: g.unit,
                        exponent: g.exponent,
                        flags: value,
                        kind: match tag {
                            8 => "input",
                            9 => "output",
                            _ => "feature",
                        },
                        report_id: g.report_id,
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
            let label = if multiple {
                format!("{app_name} · {what}")
            } else {
                what.to_owned()
            };
            rows.push((label, value));
        };
        push("Function", app_name.clone());

        let ids: std::collections::BTreeSet<u32> = app.fields.iter().map(|f| f.report_id).collect();
        if ids.iter().any(|i| *i != 0) {
            push(
                "Report IDs",
                ids.iter()
                    .map(|i| i.to_string())
                    .collect::<Vec<_>>()
                    .join(", "),
            );
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
                    leds.extend(
                        f.usages
                            .iter()
                            .filter_map(|u| led_name((*u & 0xffff) as usize - 1)),
                    );
                }
                0x07 if variable && f.usages.len() == 8 => push(
                    "Modifier keys",
                    "8 (Ctrl, Shift, Alt, Meta × left/right)".into(),
                ),
                0x07 if !variable && f.kind == "input" => push(
                    "Key reports",
                    format!(
                        "up to {} keys pressed at once, key codes 0–{}",
                        f.count, f.max
                    ),
                ),
                p if p > 0x100 => vendor_bits += f.size * f.count,
                0x0c if !variable && f.kind == "input" => {
                    push(
                        "Media / consumer keys",
                        format!("up to {} at once (key codes 0–{})", f.count, f.max),
                    );
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
                            let inches = if f.unit & 0xff == 0x13 {
                                span
                            } else {
                                span / 2.54
                            };
                            if inches > 0.0 {
                                text += &format!(
                                    " · ~{:.0} counts/inch",
                                    (f.max - f.min) as f64 / inches
                                );
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
            push(
                "Vendor-defined data",
                format!("{vendor_bits} bits per report (proprietary protocol)"),
            );
        }
    }
    if apps.is_empty() {
        rows.push(("Contents".into(), "Could not be decoded".into()));
    }
    rows
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The standard 3-button boot-protocol mouse from the HID specification.
    const MOUSE: &[u8] = &[
        0x05, 0x01, 0x09, 0x02, 0xA1, 0x01, 0x09, 0x01, 0xA1, 0x00, 0x05, 0x09, 0x19, 0x01, 0x29,
        0x03, 0x15, 0x00, 0x25, 0x01, 0x95, 0x03, 0x75, 0x01, 0x81, 0x02, 0x95, 0x01, 0x75, 0x05,
        0x81, 0x03, 0x05, 0x01, 0x09, 0x30, 0x09, 0x31, 0x09, 0x38, 0x15, 0x81, 0x25, 0x7F, 0x75,
        0x08, 0x95, 0x03, 0x81, 0x06, 0xC0, 0xC0,
    ];

    /// The standard boot-protocol keyboard.
    const KEYBOARD: &[u8] = &[
        0x05, 0x01, 0x09, 0x06, 0xA1, 0x01, 0x05, 0x07, 0x19, 0xE0, 0x29, 0xE7, 0x15, 0x00, 0x25,
        0x01, 0x75, 0x01, 0x95, 0x08, 0x81, 0x02, 0x95, 0x01, 0x75, 0x08, 0x81, 0x03, 0x95, 0x05,
        0x75, 0x01, 0x05, 0x08, 0x19, 0x01, 0x29, 0x05, 0x91, 0x02, 0x95, 0x01, 0x75, 0x03, 0x91,
        0x03, 0x95, 0x06, 0x75, 0x08, 0x15, 0x00, 0x25, 0x65, 0x05, 0x07, 0x19, 0x00, 0x29, 0x65,
        0x81, 0x00, 0xC0,
    ];

    fn row(rows: &[(String, String)], label: &str) -> Option<String> {
        rows.iter()
            .find(|(l, _)| l == label)
            .map(|(_, v)| v.clone())
    }

    #[test]
    fn decodes_a_mouse() {
        let rows = decode_hid(MOUSE);
        assert_eq!(row(&rows, "Function").as_deref(), Some("Mouse"));
        assert_eq!(row(&rows, "Buttons").as_deref(), Some("3"));
        assert_eq!(
            row(&rows, "X axis").as_deref(),
            Some("8-bit relative, -127…127")
        );
        assert_eq!(
            row(&rows, "Y axis").as_deref(),
            Some("8-bit relative, -127…127")
        );
        assert_eq!(
            row(&rows, "Vertical wheel").as_deref(),
            Some("8-bit relative, -127…127")
        );
    }

    #[test]
    fn decodes_a_keyboard() {
        let rows = decode_hid(KEYBOARD);
        assert_eq!(row(&rows, "Function").as_deref(), Some("Keyboard"));
        assert!(row(&rows, "Modifier keys").unwrap().starts_with("8 "));
        assert!(row(&rows, "Key reports").unwrap().contains("up to 6 keys"));
        assert_eq!(
            row(&rows, "Indicator LEDs").as_deref(),
            Some("Num Lock, Caps Lock, Scroll Lock, Compose, Kana")
        );
    }

    #[test]
    fn global_state_is_restored_by_pop() {
        // push; change the report size; pop → the size is back to what it was
        let apps = parse_hid(&[
            0x05, 0x01, 0x09, 0x02, 0xA1, 0x01, 0x75, 0x08, 0xA4, 0x75, 0x10, 0xB4, 0x95, 0x01,
            0x09, 0x30, 0x81, 0x06, 0xC0,
        ]);
        assert_eq!(apps[0].fields[0].size, 8);
    }

    #[test]
    fn empty_or_broken_descriptors_do_not_panic() {
        assert_eq!(decode_hid(&[]).len(), 1); // "Contents: could not be decoded"
        let _ = decode_hid(&[0xFE, 0xFF]); // long item header cut short
        let _ = decode_hid(&[0x05]); // item data missing
    }
}
