use std::collections::{HashMap, HashSet};

use super::block::{is_removable, parse_lsblk, stack, Block, LSBLK_COLUMNS};
use super::kind::{classify, describe, Kind};
use super::model::{FilesystemRow, Overview, Snapshot};
use super::mounts::{parse_mountinfo, Mount};
use super::usage::{parse_df, Usage, DF_COLUMNS};
use crate::common::cmd::run;
use crate::common::format::format_bytes;
use crate::common::{Detail, Details};
use crate::error::{AppError, Result};

struct Entry {
    dev: String,
    kind: Kind,
    row: FilesystemRow,
}

/// Count each filesystem once: bind mounts and btrfs subvolumes share a device number.
fn overview(entries: &[Entry]) -> Overview {
    let mut seen = HashSet::new();
    let mut o = Overview::default();
    for e in entries.iter().filter(|e| e.kind.is_storage()) {
        let (Some(size), true) = (e.row.size, seen.insert(&e.dev)) else {
            continue;
        };
        o.size += size;
        o.used += e.row.used.unwrap_or(0);
        o.available += e.row.available.unwrap_or(0);
        o.volumes += 1;
    }
    o
}

fn percent(part: u64, whole: u64) -> String {
    format!("{:.0}%", part as f64 / whole.max(1) as f64 * 100.0)
}

fn drive_text(disk: &Block) -> String {
    let kind = match (disk.transport.as_deref(), disk.rotational) {
        (Some("nvme"), _) => "NVMe solid-state drive",
        (_, true) => "Hard disk",
        _ => "Solid-state drive",
    };
    [disk.model.clone(), Some(kind.to_owned())]
        .into_iter()
        .flatten()
        .collect::<Vec<_>>()
        .join(" · ")
}

fn details(m: &Mount, kind: Kind, usage: &Usage, chain: &[&Block], read_only: bool) -> Vec<Detail> {
    let mut d = Details::new();
    d.add("Filesystem", "Type", &m.fstype);
    d.add_opt("Filesystem", "About", describe(&m.fstype));
    d.add("Filesystem", "Storage", kind.label());
    d.add("Filesystem", "Mounted at", &m.target);
    d.add("Filesystem", "Source", &m.source);
    if m.root != "/" {
        d.add("Filesystem", "Folder of source", &m.root);
    }
    d.add(
        "Filesystem",
        "Access",
        if read_only {
            "Read-only"
        } else {
            "Read and write"
        },
    );

    if let Some(size) = usage.size {
        let used = usage.used.unwrap_or(0);
        let free = usage.available.unwrap_or(0);
        d.add("Space", "Size", format_bytes(size));
        d.add(
            "Space",
            "Used",
            format!("{} ({})", format_bytes(used), percent(used, used + free)),
        );
        d.add("Space", "Available", format_bytes(free));
        let reserved = size.saturating_sub(used + free);
        if reserved > 0 {
            d.add(
                "Space",
                "Reserved",
                format!("{} (kept for the system)", format_bytes(reserved)),
            );
        }
    }
    if let (Some(total), Some(used)) = (usage.inodes_total, usage.inodes_used) {
        d.add("Files", "File slots", total.to_string());
        d.add(
            "Files",
            "In use",
            format!("{used} ({})", percent(used, total)),
        );
    }

    if let Some(top) = chain.first() {
        d.add_opt("Device", "UUID", top.uuid.clone());
        d.add_opt("Device", "Label", top.label.clone());
        if chain.len() > 1 {
            d.add(
                "Device",
                "Built on",
                chain
                    .iter()
                    .map(|b| format!("{} ({})", b.name, b.kind))
                    .collect::<Vec<_>>()
                    .join(" → "),
            );
        }
        if let Some(disk) = chain.iter().rev().find(|b| b.kind == "disk") {
            d.add("Device", "Drive", drive_text(disk));
            d.add_opt("Device", "Connection", disk.transport.clone());
        }
    }

    d.add("Options", "Mount options", m.options.join(", "));
    if !m.super_options.is_empty() {
        d.add("Options", "Filesystem options", m.super_options.join(", "));
    }
    d.add("Options", "Device number", &m.dev);
    d.finish()
}

fn resolve_block(source: &str) -> Option<String> {
    if !source.starts_with("/dev/") {
        return None;
    }
    std::fs::canonicalize(source)
        .ok()?
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
}

/// A hung share must not freeze the page, so each is asked with a short deadline.
fn network_usage(target: &str) -> Option<Usage> {
    let text = run("timeout", &["2", "df", "-B1", DF_COLUMNS, "--", target])?;
    parse_df(&text).into_values().next()
}

fn entries() -> Result<Vec<Entry>> {
    let mountinfo = std::fs::read_to_string("/proc/self/mountinfo")
        .map_err(|e| AppError::Other(format!("Couldn't read the mount table: {e}")))?;
    let mounts = parse_mountinfo(&mountinfo);
    let blocks = run("lsblk", &["-J", "-l", "-o", LSBLK_COLUMNS])
        .map(|t| parse_lsblk(&t))
        .unwrap_or_default();
    let local: HashMap<String, Usage> = run("df", &["-B1", "-a", "-l", DF_COLUMNS])
        .map(|t| parse_df(&t))
        .unwrap_or_default();

    let mut out: Vec<Entry> = mounts
        .iter()
        .map(|m| {
            let chain = resolve_block(&m.source)
                .map(|k| stack(&blocks, &k))
                .unwrap_or_default();
            let removable = is_removable(&chain)
                || (chain.is_empty()
                    && (m.target.starts_with("/run/media/") || m.target.starts_with("/media/")));
            let kind = classify(&m.fstype, &m.source, removable);
            let usage = if kind == Kind::Network {
                network_usage(&m.target).unwrap_or_default()
            } else {
                local.get(&m.target).copied().unwrap_or_default()
            };
            let read_only = m.options.iter().any(|o| o == "ro");
            Entry {
                dev: m.dev.clone(),
                kind,
                row: FilesystemRow {
                    mount: m.target.clone(),
                    source: m.source.clone(),
                    fstype: m.fstype.clone(),
                    kind: kind.as_str().to_owned(),
                    read_only,
                    size: usage.size,
                    used: usage.used,
                    available: usage.available,
                    inodes_total: usage.inodes_total,
                    inodes_used: usage.inodes_used,
                    label: chain.first().and_then(|b| b.label.clone()),
                    details: details(m, kind, &usage, &chain, read_only),
                },
            }
        })
        .collect();
    out.sort_by(|a, b| (a.kind.rank(), &a.row.mount).cmp(&(b.kind.rank(), &b.row.mount)));
    Ok(out)
}

pub(crate) fn snapshot() -> Result<Snapshot> {
    let entries = entries()?;
    Ok(Snapshot {
        overview: overview(&entries),
        filesystems: entries.into_iter().map(|e| e.row).collect(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(dev: &str, kind: Kind, size: Option<u64>, used: u64) -> Entry {
        Entry {
            dev: dev.to_owned(),
            kind,
            row: FilesystemRow {
                mount: format!("/{dev}"),
                source: String::new(),
                fstype: String::new(),
                kind: kind.as_str().to_owned(),
                read_only: false,
                size,
                used: size.map(|_| used),
                available: size.map(|s| s - used),
                inodes_total: None,
                inodes_used: None,
                label: None,
                details: Vec::new(),
            },
        }
    }

    #[test]
    fn totals_count_a_shared_device_once_and_skip_memory_and_virtual() {
        let all = [
            entry("259:2", Kind::Disk, Some(1000), 400),
            entry("259:2", Kind::Disk, Some(1000), 400),
            entry("8:17", Kind::Removable, Some(500), 100),
            entry("0:30", Kind::Memory, Some(900), 0),
            entry("0:4", Kind::Virtual, None, 0),
        ];
        assert_eq!(
            overview(&all),
            Overview {
                size: 1500,
                used: 500,
                available: 1000,
                volumes: 2
            }
        );
    }

    #[test]
    fn percentages_are_whole_numbers_and_safe_for_empty_volumes() {
        assert_eq!(percent(1, 4), "25%");
        assert_eq!(percent(0, 0), "0%");
    }

    #[test]
    fn the_detail_list_names_the_stack_and_the_reserved_space() {
        let m =
            &parse_mountinfo("26 1 259:2 / / rw,relatime - ext4 /dev/x rw,errors=remount-ro")[0];
        let usage = Usage {
            size: Some(1000),
            used: Some(400),
            available: Some(500),
            inodes_total: Some(10),
            inodes_used: Some(5),
        };
        let d = details(m, Kind::Disk, &usage, &[], false);
        let json = serde_json::to_value(&d).unwrap();
        let get = |label: &str| {
            json.as_array()
                .unwrap()
                .iter()
                .find(|x| x["label"] == label)
                .and_then(|x| x["value"].as_str())
        };
        assert_eq!(get("Access"), Some("Read and write"));
        assert!(get("Reserved").unwrap().starts_with("100 B"));
        assert_eq!(get("In use"), Some("5 (50%)"));
    }
}
