use super::pci::pci_rows;
use crate::common::ids::*;
use crate::common::sysfs::*;
use crate::common::Details;

pub(crate) fn report(d: &mut Details, connector: &str) {
    let Some(dir) = std::fs::read_dir("/sys/class/drm").ok().and_then(|it| {
        it.flatten().map(|e| e.path()).find(|p| {
            p.file_name()
                .map(|n| {
                    n.to_string_lossy()
                        .split_once('-')
                        .map(|(_, c)| c == connector)
                        .unwrap_or(false)
                })
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

    let card = dir.join("device");
    let pci = if card.join("vendor").exists() {
        card
    } else {
        card.join("device")
    };
    pci_rows(d, "Graphics adapter", &pci);
}

fn edid_extra(d: &mut Details, raw: &[u8]) {
    if raw.len() < 128 {
        return;
    }
    let s = "Panel (from EDID)";
    d.add(s, "EDID version", format!("{}.{}", raw[18], raw[19]));

    let id = u16::from_be_bytes([raw[8], raw[9]]);
    let code: String = [id >> 10, id >> 5, id]
        .iter()
        .map(|v| (b'A' + (*v as u8 & 0x1f) - 1) as char)
        .collect();
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
        d.add(
            s,
            "Signal",
            format!("Digital · {interface} · {depth} colour"),
        );
    } else {
        d.add(s, "Signal", "Analog");
    }
    if raw[23] != 0xff {
        d.add(
            s,
            "Gamma",
            format!("{:.2}", (raw[23] as f64 + 100.0) / 100.0),
        );
    }

    for block in raw[54..126].chunks(18) {
        let clock = u16::from_le_bytes([block[0], block[1]]) as f64 * 10_000.0;
        if clock > 0.0 {
            let h = block[2] as u32 | ((block[4] as u32 & 0xf0) << 4);
            let hb = block[3] as u32 | ((block[4] as u32 & 0x0f) << 8);
            let v = block[5] as u32 | ((block[7] as u32 & 0xf0) << 4);
            let vb = block[6] as u32 | ((block[7] as u32 & 0x0f) << 8);
            let refresh = clock / ((h + hb) as f64 * (v + vb) as f64);
            d.add(
                s,
                "Preferred mode",
                format!(
                    "{h} × {v} @ {refresh:.2} Hz · {:.1} MHz pixel clock",
                    clock / 1e6
                ),
            );
        } else if block[3] == 0xfd {
            d.add(
                s,
                "Refresh range",
                format!(
                    "{}–{} Hz vertical · {}–{} kHz horizontal",
                    block[5], block[6], block[7], block[8]
                ),
            );
            if block[9] != 0 {
                d.add(
                    s,
                    "Max pixel clock",
                    format!("{} MHz", block[9] as u32 * 10),
                );
            }
        }
    }
    d.add(
        s,
        "EDID blocks",
        format!("{} (extension blocks: {})", raw.len() / 128, raw[126]),
    );
}
