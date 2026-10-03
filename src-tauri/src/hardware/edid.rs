use crate::common::sysfs::*;
use crate::common::{Detail, Details};

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
    let manufacturer: String = [letter(id >> 10), letter(id >> 5), letter(id)]
        .iter()
        .collect();

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

#[cfg(target_os = "linux")]
pub fn connected_edids() -> Vec<(String, Edid)> {
    let mut out = Vec::new();
    let Ok(entries) = std::fs::read_dir("/sys/class/drm") else {
        return out;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        let file = entry.file_name().to_string_lossy().into_owned();
        let Some((_, connector)) = file.split_once('-') else {
            continue;
        };
        if read(path.join("status")).as_deref() != Some("connected") {
            continue;
        }
        if let Some(edid) = std::fs::read(path.join("edid"))
            .ok()
            .and_then(|b| parse_edid(&b))
        {
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
        d.add(
            "Monitor",
            "Product code",
            format!("0x{:04x}", e.product_code),
        );
        if e.year > 1990 {
            d.add(
                "Monitor",
                "Manufactured",
                format!("Week {}, {}", e.week, e.year),
            );
        }
        if e.width_cm > 0 && e.height_cm > 0 {
            let (w, h) = (e.width_cm as f64, e.height_cm as f64);
            d.add(
                "Panel",
                "Physical size",
                format!(
                    "{} × {} cm · {:.1}″ diagonal",
                    e.width_cm,
                    e.height_cm,
                    (w * w + h * h).sqrt() / 2.54
                ),
            );
            d.add(
                "Panel",
                "Pixel density",
                format!("{:.0} PPI", f.width as f64 / (w / 2.54)),
            );
        }
    }

    d.add(
        "Display",
        "Resolution",
        format!("{} × {}", f.width, f.height),
    );
    d.add("Display", "Scale factor", format!("{}×", f.scale_factor));
    d.add("Display", "Position", format!("{}, {}", f.x, f.y));
    d.add("Display", "Primary", if f.primary { "Yes" } else { "No" });
    d.add_opt("Connection", "Connector", f.connector);
    d.finish()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> Vec<u8> {
        let mut e = vec![0u8; 128];
        e[..8].copy_from_slice(&[0, 255, 255, 255, 255, 255, 255, 0]);
        e[8..10].copy_from_slice(&[0x61, 0xA9]);
        e[10..12].copy_from_slice(&[0x13, 0xB0]);
        e[16] = 34;
        e[17] = 34;
        e[21] = 60;
        e[22] = 34;
        e[57] = 0xFC;
        e[59..59 + 12].copy_from_slice(b"P27FBB-RGGL\n");
        e
    }

    #[test]
    fn reads_identity_from_the_block() {
        let edid = parse_edid(&sample()).expect("valid block");
        assert_eq!(edid.model(), Some("P27FBB-RGGL"));
        assert_eq!(edid.width_cm(), Some(60));
        assert_eq!(edid.manufacturer, "XMI");
        assert_eq!(edid.product_code, 0xB013);
        assert_eq!(edid.year, 2024);
    }

    #[test]
    fn rejects_garbage() {
        assert!(parse_edid(&[0u8; 128]).is_none());
        assert!(parse_edid(&[1, 2, 3]).is_none());
    }

    #[test]
    fn known_makers_get_friendly_names() {
        assert_eq!(maker_name("XMI"), Some("Xiaomi"));
        assert_eq!(maker_name("SAM"), Some("Samsung"));
        assert_eq!(maker_name("???"), None);
    }
}
