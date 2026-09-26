//! Graphics cards.

use super::model::*;
use super::pci::{pci_model, Pci};
use crate::common::format::*;
use crate::common::nvidia::*;
use crate::common::sysfs::*;
use crate::common::Details;

pub(crate) fn gpus(pci: &Pci) -> Vec<Component> {
    let smi = nvidia_smi();
    let mut seen: Vec<std::path::PathBuf> = Vec::new();
    let mut out = Vec::new();

    for e in std::fs::read_dir("/sys/class/drm")
        .into_iter()
        .flatten()
        .flatten()
    {
        let card = e.file_name().to_string_lossy().into_owned();
        if !card.starts_with("card") || card.contains('-') {
            continue;
        }
        let dev = e.path().join("device");
        let Ok(real) = std::fs::canonicalize(&dev) else {
            continue;
        };
        if seen.contains(&real) {
            continue;
        }
        seen.push(real.clone());
        let slot = real
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default();
        let Some(p) = pci.at(&slot) else { continue };

        let mine = smi
            .iter()
            .find(|s| s.values.first().map(|b| b.to_lowercase().ends_with(&slot)) == Some(true));
        let val = |i: usize| {
            mine.and_then(|s| s.values.get(i))
                .filter(|v| !v.contains("N/A"))
                .cloned()
        };
        let model = val(1).unwrap_or_else(|| pci_model(p));
        let driver = std::fs::read_link(dev.join("driver"))
            .ok()
            .and_then(|l| l.file_name().map(|n| n.to_string_lossy().into_owned()));

        let mut d = Details::new();
        d.add("Graphics card", "Model", &model);
        d.add_opt("Graphics card", "Chip", p.device_name.clone());
        d.add_opt("Graphics card", "Vendor", p.vendor_name.clone());
        d.add_opt(
            "Graphics card",
            "Driver",
            driver.clone().map(|dr| match val(10) {
                Some(v) => format!("{dr} {v}"),
                None => dr,
            }),
        );
        d.add_opt("Graphics card", "Video BIOS", val(11));
        d.add("Graphics card", "PCI address", &slot);

        let vram = val(2)
            .and_then(|v| v.parse::<u64>().ok())
            .map(|m| m << 20)
            .or_else(|| read(dev.join("mem_info_vram_total")).and_then(|v| v.parse().ok()));
        d.add_opt("Memory", "Video memory", vram.map(format_bytes));
        d.add_opt(
            "Memory",
            "In use",
            val(3)
                .and_then(|v| v.parse::<u64>().ok())
                .map(|m| format_bytes(m << 20)),
        );

        let gen = |cur: Option<String>, w: Option<String>| match (cur, w) {
            (Some(g), Some(w)) => Some(format!("PCIe {g}.0 ×{w}")),
            _ => None,
        };
        d.add_opt(
            "PCIe link",
            "Now",
            gen(val(14), val(16)).or_else(|| {
                Some(format!(
                    "{} ×{}",
                    read(dev.join("current_link_speed"))?,
                    read(dev.join("current_link_width"))?
                ))
            }),
        );
        d.add_opt(
            "PCIe link",
            "Maximum",
            gen(val(15), val(17)).or_else(|| {
                Some(format!(
                    "{} ×{}",
                    read(dev.join("max_link_speed"))?,
                    read(dev.join("max_link_width"))?
                ))
            }),
        );

        d.add_opt(
            "Live readings",
            "Temperature",
            val(4).map(|t| format!("{t} °C")),
        );
        d.add_opt(
            "Live readings",
            "Power draw",
            match (val(5), val(6)) {
                (Some(a), Some(b)) => Some(format!("{a} W of {b} W limit")),
                _ => None,
            },
        );
        d.add_opt(
            "Live readings",
            "Core clock",
            match (val(7), val(8)) {
                (Some(a), Some(b)) => Some(format!("{a} MHz (max {b} MHz)")),
                _ => None,
            },
        );
        d.add_opt(
            "Live readings",
            "Memory clock",
            val(9).map(|c| format!("{c} MHz")),
        );
        d.add_opt("Live readings", "Fan", val(12).map(|f| format!("{f} %")));
        d.add_opt("Live readings", "Load", val(13).map(|f| format!("{f} %")));

        let connected: Vec<String> = std::fs::read_dir("/sys/class/drm")
            .into_iter()
            .flatten()
            .flatten()
            .filter_map(|c| {
                let n = c.file_name().to_string_lossy().into_owned();
                let rest = n.strip_prefix(&format!("{card}-"))?.to_owned();
                (read(c.path().join("status")).as_deref() == Some("connected")).then_some(rest)
            })
            .collect();
        d.add(
            "Outputs",
            "Connected displays",
            if connected.is_empty() {
                "None".into()
            } else {
                connected.join(", ")
            },
        );

        out.push(Component {
            id: format!("sys:gpu:{card}"),
            kind: "gpu",
            name: model,
            subtitle: Some(
                [vram.map(format_bytes), p.vendor_name.clone()]
                    .into_iter()
                    .flatten()
                    .collect::<Vec<_>>()
                    .join(" · "),
            ),
            details: d.finish(),
        });
    }
    out
}
