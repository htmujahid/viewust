//! The computer as a whole.

use super::pci::pci_rows;
use crate::common::format::*;
use crate::common::sysfs::*;
use crate::common::Details;
use std::path::Path;

#[cfg(target_os = "linux")]
pub(crate) fn report(d: &mut Details) {
    // graphics adapters
    let mut seen = Vec::new();
    for e in std::fs::read_dir("/sys/class/drm")
        .into_iter()
        .flatten()
        .flatten()
    {
        let name = e.file_name().to_string_lossy().into_owned();
        if name.starts_with("card") && !name.contains('-') {
            let dev = e.path().join("device");
            if let Ok(real) = std::fs::canonicalize(&dev) {
                if !seen.contains(&real) {
                    seen.push(real);
                    pci_rows(d, &format!("Graphics adapter · {name}"), &dev);
                }
            }
        }
    }

    // processor details beyond the basics
    let cpu = Path::new("/sys/devices/system/cpu/cpu0");
    let khz = |f: &str| read(cpu.join("cpufreq").join(f)).and_then(|v| v.parse::<f64>().ok());
    if let (Some(lo), Some(hi)) = (khz("cpuinfo_min_freq"), khz("cpuinfo_max_freq")) {
        d.add(
            "Processor clock",
            "Range",
            format!("{:.1} – {:.1} GHz", lo / 1e6, hi / 1e6),
        );
    }
    d.add_opt(
        "Processor clock",
        "Scaling driver",
        read(cpu.join("cpufreq/scaling_driver")),
    );
    d.add_opt(
        "Processor clock",
        "Governor",
        read(cpu.join("cpufreq/scaling_governor")),
    );
    for i in 0..6 {
        let c = cpu.join(format!("cache/index{i}"));
        if let (Some(level), Some(kind), Some(size)) = (
            read(c.join("level")),
            read(c.join("type")),
            read(c.join("size")),
        ) {
            d.add(
                "Processor cache",
                format!("L{level} {}", kind.to_lowercase()),
                size,
            );
        }
    }

    // storage volumes
    for disk in sysinfo::Disks::new_with_refreshed_list()
        .iter()
        .filter(|x| x.total_space() > 0)
    {
        d.add(
            "Storage volumes",
            disk.mount_point().to_string_lossy().into_owned(),
            format!(
                "{} · {:?} · {} total, {} free",
                disk.file_system().to_string_lossy(),
                disk.kind(),
                format_bytes(disk.total_space()),
                format_bytes(disk.available_space())
            ),
        );
    }

    // network adapters
    for (name, data) in sysinfo::Networks::new_with_refreshed_list()
        .iter()
        .filter(|(n, _)| n.as_str() != "lo")
    {
        let ips: Vec<String> = data
            .ip_networks()
            .iter()
            .map(|ip| ip.addr.to_string())
            .collect();
        d.add(
            "Network adapters",
            name.clone(),
            format!(
                "{} · {}",
                data.mac_address(),
                if ips.is_empty() {
                    "no address".into()
                } else {
                    ips.join(", ")
                }
            ),
        );
    }
}
