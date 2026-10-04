use super::pci::pci_rows;
use crate::common::sysfs::*;
use crate::common::Details;
use std::path::PathBuf;
use std::process::Command;

pub(crate) fn report(d: &mut Details, rest: &str) {
    let Some(card) = rest.split('-').next().and_then(|c| c.parse::<u32>().ok()) else {
        return;
    };
    let section = "Sound card";
    d.add_opt(
        section,
        "Card ID",
        read(format!("/proc/asound/card{card}/id")),
    );
    if let Ok(cards) = std::fs::read_to_string("/proc/asound/cards") {
        let mut lines = cards.lines();
        while let Some(line) = lines.next() {
            if line.trim_start().starts_with(&format!("{card} [")) {
                d.add_opt(
                    section,
                    "Description",
                    lines.next().map(|l| l.trim().to_owned()),
                );
            }
        }
    }

    if let Ok(codec) = std::fs::read_to_string(format!("/proc/asound/card{card}/codec#0")) {
        for line in codec.lines().take(8) {
            if let Some((k, v)) = line.split_once(':') {
                if matches!(
                    k,
                    "Codec" | "Address" | "Vendor Id" | "Subsystem Id" | "Revision Id"
                ) {
                    d.add("Audio codec chip", k, v.trim());
                }
            }
        }
    }

    if let Ok(pcm) = std::fs::read_to_string("/proc/asound/pcm") {
        for line in pcm
            .lines()
            .filter(|l| l.starts_with(&format!("{card:02}-")))
        {
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

    if let Ok(out) = Command::new("amixer")
        .args(["-c", &card.to_string(), "scontrols"])
        .output()
    {
        let names: Vec<String> = String::from_utf8_lossy(&out.stdout)
            .lines()
            .filter_map(|l| l.split('\'').nth(1).map(str::to_owned))
            .filter(|n| n.contains("Mic") || n.contains("Capture") || n.contains("Headphone"))
            .collect();
        if !names.is_empty() {
            d.add("Streams", "Related mixer controls", names.join(", "));
        }
    }

    pci_rows(
        d,
        "Audio controller",
        &PathBuf::from(format!("/sys/class/sound/card{card}/device")),
    );
}
