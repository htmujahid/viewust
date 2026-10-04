use std::collections::HashMap;

use super::model::OsMemory;
use crate::common::format::format_bytes;
use crate::common::sysfs::read;
use crate::common::Details;

/// `/proc/meminfo` lines: `MemTotal:       32795828 kB`. Values come back in bytes.
pub(crate) fn parse_meminfo(text: &str) -> HashMap<String, u64> {
    text.lines()
        .filter_map(|line| {
            let (key, rest) = line.split_once(':')?;
            let value: u64 = rest.split_whitespace().next()?.parse().ok()?;
            let bytes = if rest.trim_end().ends_with("kB") {
                value * 1024
            } else {
                value
            };
            Some((key.trim().to_owned(), bytes))
        })
        .collect()
}

pub(crate) struct SwapDevice {
    pub(crate) name: String,
    pub(crate) kind: String,
    pub(crate) size: u64,
    pub(crate) used: u64,
    pub(crate) priority: String,
}

/// `/proc/swaps` is a header line, then one device per line with sizes in KiB.
pub(crate) fn parse_swaps(text: &str) -> Vec<SwapDevice> {
    text.lines()
        .skip(1)
        .filter_map(|line| {
            let mut f = line.split_whitespace();
            Some(SwapDevice {
                name: f.next()?.to_owned(),
                kind: f.next()?.to_owned(),
                size: f.next()?.parse::<u64>().ok()? * 1024,
                used: f.next()?.parse::<u64>().ok()? * 1024,
                priority: f.next()?.to_owned(),
            })
        })
        .collect()
}

pub(crate) fn overcommit(value: &str) -> String {
    match value.trim() {
        "0" => "Heuristic (0): large askings are refused".into(),
        "1" => "Always allow (1)".into(),
        "2" => "Strict (2): never promise more than exists".into(),
        other => other.to_owned(),
    }
}

pub(crate) fn snapshot() -> OsMemory {
    let info = std::fs::read_to_string("/proc/meminfo")
        .map(|t| parse_meminfo(&t))
        .unwrap_or_default();
    let get = |k: &str| info.get(k).copied();
    let val = |k: &str| get(k).map(format_bytes);
    let total = get("MemTotal").unwrap_or(0);
    let available = get("MemAvailable").unwrap_or(0);
    let swap_total = get("SwapTotal").unwrap_or(0);
    let swap_used = swap_total.saturating_sub(get("SwapFree").unwrap_or(0));

    let mut d = Details::new();
    d.add_opt("Memory", "Total", val("MemTotal"));
    d.add(
        "Memory",
        "In use",
        format_bytes(total.saturating_sub(available)),
    );
    d.add_opt("Memory", "Available", val("MemAvailable"));
    d.add_opt("Memory", "Truly free", val("MemFree"));
    d.add_opt("Memory", "Disk cache", val("Cached"));
    d.add_opt("Memory", "Buffers", val("Buffers"));
    d.add_opt("Memory", "Shared", val("Shmem"));
    d.add_opt("Memory", "Locked in RAM", val("Mlocked"));

    for s in std::fs::read_to_string("/proc/swaps")
        .map(|t| parse_swaps(&t))
        .unwrap_or_default()
    {
        d.add(
            "Swap",
            &s.name,
            format!(
                "{} {} · {} used · priority {}",
                format_bytes(s.size),
                s.kind,
                format_bytes(s.used),
                s.priority
            ),
        );
    }
    d.add_opt(
        "Swap",
        "Swappiness",
        read("/proc/sys/vm/swappiness")
            .map(|v| format!("{v} of 200: how eagerly RAM is swapped out")),
    );
    d.add_opt(
        "Swap",
        "Zswap",
        read("/sys/module/zswap/parameters/enabled").map(|v| {
            if v == "Y" {
                "Enabled (compressed swap cache in RAM)".to_owned()
            } else {
                "Disabled".to_owned()
            }
        }),
    );

    d.add_opt("Kernel's own use", "Slab", val("Slab"));
    d.add_opt("Kernel's own use", "Reclaimable", val("SReclaimable"));
    d.add_opt("Kernel's own use", "Page tables", val("PageTables"));
    d.add_opt("Kernel's own use", "Kernel stacks", val("KernelStack"));
    d.add_opt("Kernel's own use", "Waiting to be written", val("Dirty"));
    d.add_opt("Kernel's own use", "Being written now", val("Writeback"));

    d.add_opt("Huge pages", "Anonymous huge pages", val("AnonHugePages"));
    d.add_opt(
        "Huge pages",
        "Transparent huge pages",
        read("/sys/kernel/mm/transparent_hugepage/enabled")
            .and_then(|t| super::info::bracketed(&t)),
    );
    d.add_opt(
        "Huge pages",
        "Reserved huge pages",
        get("HugePages_Total")
            .filter(|n| *n > 0)
            .map(|n| n.to_string()),
    );

    d.add_opt(
        "Settings",
        "Overcommit",
        read("/proc/sys/vm/overcommit_memory").map(|v| overcommit(&v)),
    );
    d.add_opt(
        "Settings",
        "Memory map limit",
        read("/proc/sys/vm/max_map_count"),
    );

    OsMemory {
        total,
        used: total.saturating_sub(available),
        available,
        swap_total,
        swap_used,
        details: d.finish(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const MEMINFO: &str = "\
MemTotal:       32795828 kB
MemFree:         1130048 kB
MemAvailable:   22893744 kB
Cached:         20275968 kB
SwapTotal:      15624188 kB
SwapFree:       15624188 kB
HugePages_Total:       0
";

    #[test]
    fn meminfo_values_come_back_in_bytes() {
        let m = parse_meminfo(MEMINFO);
        assert_eq!(m["MemTotal"], 32_795_828 * 1024);
        assert_eq!(m["HugePages_Total"], 0);
        assert_eq!(m.len(), 7);
    }

    #[test]
    fn swap_devices_give_their_size_use_and_priority() {
        let s = parse_swaps(
            "Filename  Type  Size  Used  Priority\n/dev/sdc1 partition 15624188 4096 -2\n",
        );
        assert_eq!(s.len(), 1);
        assert_eq!(s[0].name, "/dev/sdc1");
        assert_eq!(s[0].size, 15_624_188 * 1024);
        assert_eq!(s[0].used, 4096 * 1024);
        assert_eq!(s[0].priority, "-2");
    }

    #[test]
    fn overcommit_modes_are_named() {
        assert!(overcommit("0").starts_with("Heuristic"));
        assert!(overcommit("2\n").starts_with("Strict"));
        assert_eq!(overcommit("9"), "9");
    }

    #[test]
    fn this_machine_reports_its_memory() {
        let m = snapshot();
        assert!(m.total > 0 && m.available > 0 && m.used > 0);
        assert!(!m.details.is_empty());
    }
}
