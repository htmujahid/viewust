//! Extra, longer detail for a component's own page.

use super::pci::{pci_class_name, pci_model, Pci};
use super::{cpu::cpuinfo, memory::meminfo};
use crate::common::cmd::run;
use crate::common::format::*;
use crate::common::sysfs::*;
use crate::common::Details;

/// Extra, longer detail for a component's own page.
pub fn report(d: &mut Details, id: &str) {
    if id == "board" {
        let pci = Pci::load();
        for p in &pci.0 {
            d.add(
                "PCI devices",
                &p.slot,
                format!("{} · {}", pci_model(p), pci_class_name(p.class)),
            );
        }
    } else if id == "cpu" {
        let info = cpuinfo();
        if let Some(flags) = info.get("flags") {
            d.add(
                "All CPU flags",
                format!("{} flags", flags.split_whitespace().count()),
                flags.clone(),
            );
        }
        if let Ok(rd) = std::fs::read_dir("/sys/devices/system/cpu/vulnerabilities") {
            let mut rows: Vec<(String, String)> = rd
                .flatten()
                .filter_map(|e| {
                    Some((
                        e.file_name().to_string_lossy().into_owned(),
                        read(e.path())?,
                    ))
                })
                .collect();
            rows.sort();
            for (k, v) in rows {
                d.add("Security mitigations", k.replace('_', " "), v);
            }
        }
        let mut lines = Vec::new();
        for i in 0..256 {
            match read(format!(
                "/sys/devices/system/cpu/cpu{i}/cpufreq/scaling_cur_freq"
            ))
            .and_then(|v| v.parse::<f64>().ok())
            {
                Some(khz) => lines.push(format!("cpu{i:<3} {:.0} MHz", khz / 1000.0)),
                None => break,
            }
        }
        if !lines.is_empty() {
            d.add("Per-thread clock now", "Threads", lines.join("\n"));
        }
    } else if id == "ram" {
        let m = meminfo();
        let mut rows: Vec<_> = m.iter().collect();
        rows.sort_by(|a, b| b.1.cmp(a.1).then(a.0.cmp(b.0)));
        for (k, v) in rows.into_iter().take(14) {
            d.add("Kernel memory counters", k.clone(), format_bytes(*v));
        }
    } else if let Some(disk) = id.strip_prefix("disk:") {
        let udev = run(
            "udevadm",
            &["info", "--query=property", &format!("--name=/dev/{disk}")],
        )
        .unwrap_or_default();
        for line in udev.lines().filter(|l| l.starts_with("ID_")) {
            if let Some((k, v)) = line.split_once('=') {
                d.add("Device properties", k.trim_start_matches("ID_"), v);
            }
        }
        let q = |f: &str| read(format!("/sys/block/{disk}/queue/{f}"));
        d.add_opt("I/O queue", "Scheduler", q("scheduler"));
        d.add_opt(
            "I/O queue",
            "Queue depth",
            read(format!("/sys/block/{disk}/device/queue_depth")),
        );
        d.add_opt(
            "I/O queue",
            "Read-ahead",
            q("read_ahead_kb").map(|v| format!("{v} KiB")),
        );
        d.add_opt(
            "I/O queue",
            "Max request",
            q("max_sectors_kb").map(|v| format!("{v} KiB")),
        );
    } else if id.starts_with("gpu:") {
        if let Some(out) = run(
            "nvidia-smi",
            &["-q", "-d", "CLOCK,POWER,TEMPERATURE,PERFORMANCE"],
        ) {
            let mut section = String::from("Driver report");
            for line in out.lines().skip(1) {
                let t = line.trim();
                if t.is_empty() || t.starts_with("GPU ") {
                    continue;
                }
                match t.split_once(':') {
                    Some((k, v)) if !v.trim().is_empty() => {
                        d.add(section.clone(), k.trim(), v.trim())
                    }
                    Some((k, _)) => section = format!("Driver report · {}", k.trim()),
                    None => {}
                }
            }
        }
    }
}
