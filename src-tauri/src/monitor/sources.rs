use crate::common::sysfs::*;
use std::collections::HashMap;

pub(crate) fn diskstats() -> HashMap<String, (u64, u64, u64)> {
    let text = std::fs::read_to_string("/proc/diskstats").unwrap_or_default();
    text.lines()
        .filter_map(|l| {
            let f: Vec<&str> = l.split_whitespace().collect();
            let name = f.get(2)?;
            if !std::path::Path::new(&format!("/sys/block/{name}")).exists() {
                return None;
            }
            if ["loop", "ram", "zram", "dm-", "sr", "md"]
                .iter()
                .any(|p| name.starts_with(p))
            {
                return None;
            }
            Some((
                name.to_string(),
                (
                    f.get(5)?.parse().ok()?,
                    f.get(9)?.parse().ok()?,
                    f.get(12)?.parse().ok()?,
                ),
            ))
        })
        .collect()
}

pub(crate) fn netstats() -> HashMap<String, (u64, u64)> {
    let mut out = HashMap::new();
    for e in std::fs::read_dir("/sys/class/net")
        .into_iter()
        .flatten()
        .flatten()
    {
        let name = e.file_name().to_string_lossy().into_owned();
        let p = e.path();
        if name == "lo" || !p.join("device").exists() {
            continue;
        }
        let n = |f: &str| read(p.join("statistics").join(f)).and_then(|v| v.parse::<u64>().ok());
        if let (Some(rx), Some(tx)) = (n("rx_bytes"), n("tx_bytes")) {
            out.insert(name, (rx, tx));
        }
    }
    out
}

pub(crate) fn default_interface() -> Option<String> {
    crate::connection::default_route().map(|r| r.interface)
}

pub(crate) fn package_temperature() -> Option<f32> {
    for e in std::fs::read_dir("/sys/class/hwmon").ok()?.flatten() {
        let p = e.path();
        let name = read(p.join("name"))?;
        if name != "coretemp" && name != "k10temp" {
            continue;
        }
        for i in 1..32 {
            let label = read(p.join(format!("temp{i}_label")));
            if matches!(
                label.as_deref(),
                Some("Package id 0") | Some("Tctl") | Some("Tdie")
            ) {
                return read(p.join(format!("temp{i}_input")))
                    .and_then(|v| v.parse::<f32>().ok())
                    .map(|v| v / 1000.0);
            }
        }
    }
    None
}
