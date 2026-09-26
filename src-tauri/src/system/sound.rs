//! Sound cards.

use super::model::*;
use super::pci::{pci_model, Pci};
use crate::common::sysfs::*;
use crate::common::Details;
use std::path::Path;

pub(crate) fn sound(pci: &Pci) -> Vec<Component> {
    // HDMI audio on a graphics card shares its PCI slot (minus the function).
    let gpu_slots: Vec<String> = pci
        .0
        .iter()
        .filter(|d| d.class >> 16 == 0x03)
        .map(|d| {
            d.slot
                .rsplit_once('.')
                .map(|(a, _)| a.to_owned())
                .unwrap_or_default()
        })
        .collect();

    let mut out = Vec::new();
    for e in std::fs::read_dir("/sys/class/sound")
        .into_iter()
        .flatten()
        .flatten()
    {
        let name = e.file_name().to_string_lossy().into_owned();
        let Some(n) = name
            .strip_prefix("card")
            .and_then(|n| n.parse::<u32>().ok())
        else {
            continue;
        };
        if Path::new(&format!("/proc/asound/card{n}/usbid")).exists() {
            continue; // USB sound devices are listed as peripherals
        }
        let dev = e.path().join("device");
        let Some(slot) = std::fs::canonicalize(&dev)
            .ok()
            .and_then(|r| r.file_name().map(|f| f.to_string_lossy().into_owned()))
        else {
            continue;
        };
        if gpu_slots.contains(
            &slot
                .rsplit_once('.')
                .map(|(a, _)| a.to_owned())
                .unwrap_or_default(),
        ) {
            continue;
        }
        let codec = std::fs::read_to_string(format!("/proc/asound/card{n}/codec#0"))
            .ok()
            .and_then(|t| {
                t.lines()
                    .find_map(|l| l.strip_prefix("Codec:").map(|c| c.trim().to_owned()))
            });
        let ctrl = pci.at(&slot);

        let mut d = Details::new();
        d.add_opt("Audio chip", "Codec", codec.clone());
        d.add_opt(
            "Audio chip",
            "Card",
            read(format!("/proc/asound/card{n}/id")),
        );
        d.add_opt("Controller", "Model", ctrl.map(pci_model));
        d.add_opt(
            "Controller",
            "Vendor",
            ctrl.and_then(|c| c.vendor_name.clone()),
        );
        d.add("Controller", "PCI address", &slot);
        d.add_opt(
            "Controller",
            "Driver",
            std::fs::read_link(dev.join("driver"))
                .ok()
                .and_then(|l| l.file_name().map(|f| f.to_string_lossy().into_owned())),
        );
        if let Ok(pcm) = std::fs::read_to_string("/proc/asound/pcm") {
            for line in pcm.lines().filter(|l| l.starts_with(&format!("{n:02}-"))) {
                let parts: Vec<&str> = line.split(" : ").collect();
                if parts.len() >= 3 {
                    d.add(
                        "Streams",
                        parts[0].trim(),
                        format!("{} · {}", parts[1].trim(), parts[2..].join(", ")),
                    );
                }
            }
        }
        out.push(Component {
            id: format!("sys:audio:{n}"),
            kind: "soundcard",
            name: codec.unwrap_or_else(|| "Sound card".into()),
            subtitle: ctrl.map(pci_model),
            details: d.finish(),
        });
    }
    out
}
