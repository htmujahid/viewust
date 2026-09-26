//! System memory, and the opt-in read of individual modules.

use super::model::*;
use crate::common::format::*;
use crate::common::Details;
use crate::error::{AppError, Result};
use std::path::Path;
use std::process::Command;

pub(crate) fn meminfo() -> std::collections::HashMap<String, u64> {
    std::fs::read_to_string("/proc/meminfo")
        .unwrap_or_default()
        .lines()
        .filter_map(|l| {
            let (k, v) = l.split_once(':')?;
            Some((
                k.to_owned(),
                v.trim().trim_end_matches(" kB").parse::<u64>().ok()? * 1024,
            ))
        })
        .collect()
}

pub(crate) fn memory() -> Component {
    let m = meminfo();
    let g = |k: &str| m.get(k).copied().unwrap_or(0);
    let total = g("MemTotal");
    let mut d = Details::new();
    d.add("Memory", "Usable", format_bytes(total));
    d.add(
        "Memory",
        "In use",
        format_bytes(total.saturating_sub(g("MemAvailable"))),
    );
    d.add("Memory", "Available", format_bytes(g("MemAvailable")));
    d.add("Memory", "Cached", format_bytes(g("Cached")));
    if g("SwapTotal") > 0 {
        d.add("Swap", "Size", format_bytes(g("SwapTotal")));
        d.add(
            "Swap",
            "In use",
            format_bytes(g("SwapTotal") - g("SwapFree")),
        );
    }
    d.add(
        "Memory modules",
        "Status",
        "Not read yet. Slot, speed, type and maker of each module are restricted to \
         administrators. Use “Read memory modules” to unlock them.",
    );
    Component {
        id: "sys:ram".into(),
        kind: "ram",
        name: "System memory".into(),
        subtitle: Some(format_bytes(total)),
        details: d.finish(),
    }
}

/// Reads each memory module with `dmidecode`, asking for administrator
/// permission through the desktop's own password prompt. Only runs when the
/// user asks for it.
pub(crate) fn read_modules() -> Result<MemoryModules> {
    let program = [
        "/usr/sbin/dmidecode",
        "/usr/bin/dmidecode",
        "/sbin/dmidecode",
    ]
    .into_iter()
    .find(|p| Path::new(p).exists())
    .ok_or(AppError::MissingTool("dmidecode"))?;
    let out = Command::new("pkexec")
        .args([program, "-t", "17"])
        .output()
        .map_err(|e| AppError::Other(format!("could not ask for permission: {e}")))?;
    if !out.status.success() {
        // pkexec exits 126 when the prompt is dismissed and 127 when authentication fails
        return Err(match out.status.code() {
            Some(126) | Some(127) => AppError::PermissionDenied,
            _ => AppError::Other("Permission was declined or the read failed".into()),
        });
    }
    Ok(parse_dmidecode(&String::from_utf8_lossy(&out.stdout)))
}

fn parse_dmidecode(text: &str) -> MemoryModules {
    let mut modules = Vec::new();
    let mut slots = 0;
    for block in text.split("\n\n").filter(|b| b.contains("Memory Device")) {
        slots += 1;
        let field = |name: &str| {
            block
                .lines()
                .find_map(|l| l.trim().strip_prefix(&format!("{name}:")))
                .map(|v| v.trim().to_owned())
                .filter(|v| {
                    !matches!(
                        v.as_str(),
                        "" | "Unknown" | "Not Specified" | "None" | "[Empty]"
                    )
                })
        };
        let Some(size) = field("Size").filter(|s| !s.contains("No Module")) else {
            continue;
        };
        let locator = field("Locator").unwrap_or_else(|| format!("Slot {slots}"));
        let kind = field("Type");
        let speed = field("Configured Memory Speed").or_else(|| field("Speed"));
        let maker = field("Manufacturer");
        let part = field("Part Number");

        let mut d = Details::new();
        d.add("Module", "Slot", &locator);
        d.add_opt("Module", "Bank", field("Bank Locator"));
        d.add("Module", "Capacity", &size);
        d.add_opt("Module", "Type", kind.clone());
        d.add_opt("Module", "Form factor", field("Form Factor"));
        d.add_opt("Module", "Rank", field("Rank"));
        d.add_opt("Speed", "Running at", field("Configured Memory Speed"));
        d.add_opt("Speed", "Rated", field("Speed"));
        d.add_opt("Speed", "Voltage", field("Configured Voltage"));
        d.add_opt("Speed", "Data width", field("Data Width"));
        d.add_opt("Maker", "Manufacturer", maker.clone());
        d.add_opt("Maker", "Part number", part.clone());
        d.add_opt("Maker", "Serial number", field("Serial Number"));

        modules.push(Component {
            id: format!("sys:ram:{}", modules.len()),
            kind: "ram",
            name: module_name(maker, part, &size, kind.as_deref()),
            subtitle: Some(format!(
                "{locator} · {size}{}",
                speed.map(|s| format!(" · {s}")).unwrap_or_default()
            )),
            details: d.finish(),
        });
    }
    MemoryModules { modules, slots }
}

/// "Kingston KF548C38-16", or "16 GB DDR5" when the module reports no maker.
fn module_name(
    maker: Option<String>,
    part: Option<String>,
    size: &str,
    kind: Option<&str>,
) -> String {
    let name = [maker, part]
        .into_iter()
        .flatten()
        .collect::<Vec<_>>()
        .join(" ");
    if name.trim().is_empty() {
        format!("{size} {}", kind.unwrap_or_default())
            .trim()
            .to_owned()
    } else {
        name.trim().to_owned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = "Handle 0x0040, DMI type 17, 40 bytes\nMemory Device\n\tSize: 16 GB\n\tForm Factor: DIMM\n\tLocator: DIMM_A1\n\tBank Locator: BANK 0\n\tType: DDR5\n\tSpeed: 4800 MT/s\n\tManufacturer: Kingston\n\tSerial Number: 1234ABCD\n\tPart Number: KF548C38-16\n\tConfigured Memory Speed: 4800 MT/s\n\nHandle 0x0041, DMI type 17, 40 bytes\nMemory Device\n\tSize: No Module Installed\n\tLocator: DIMM_A2\n\n";

    #[test]
    fn counts_slots_and_keeps_only_populated_ones() {
        let m = parse_dmidecode(SAMPLE);
        assert_eq!(m.slots, 2);
        assert_eq!(m.modules.len(), 1);
        assert_eq!(m.modules[0].id, "sys:ram:0");
        assert_eq!(m.modules[0].name, "Kingston KF548C38-16");
        assert_eq!(
            m.modules[0].subtitle.as_deref(),
            Some("DIMM_A1 · 16 GB · 4800 MT/s")
        );
    }

    #[test]
    fn unnamed_modules_fall_back_to_size_and_type() {
        assert_eq!(module_name(None, None, "16 GB", Some("DDR5")), "16 GB DDR5");
        assert_eq!(module_name(Some("Acme".into()), None, "8 GB", None), "Acme");
    }
}
