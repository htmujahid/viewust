use super::model::*;
use crate::common::sysfs::*;
use std::collections::HashMap;
use std::process::Command;

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

pub(crate) fn gpus() -> Vec<GpuSample> {
    let num = |s: &str| s.trim().parse::<f64>().ok();
    let nvidia: Vec<GpuSample> = Command::new("nvidia-smi")
        .args([
            "--query-gpu=name,utilization.gpu,memory.used,memory.total,temperature.gpu,power.draw,power.limit,clocks.gr,clocks.mem,fan.speed",
            "--format=csv,noheader,nounits",
        ])
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| {
            String::from_utf8_lossy(&o.stdout)
                .lines()
                .map(|l| {
                    let f: Vec<&str> = l.split(", ").collect();
                    let g = |i: usize| f.get(i).and_then(|v| num(v));
                    GpuSample {
                        name: f.first().unwrap_or(&"GPU").trim().to_owned(),
                        util: g(1),
                        memory_used: g(2).map(|m| (m as u64) << 20),
                        memory_total: g(3).map(|m| (m as u64) << 20),
                        temperature: g(4),
                        power: g(5),
                        power_limit: g(6),
                        core_mhz: g(7),
                        memory_mhz: g(8),
                        fan: g(9),
                    }
                })
                .collect()
        })
        .unwrap_or_default();
    if !nvidia.is_empty() {
        return nvidia;
    }

    let mut out = Vec::new();
    for e in std::fs::read_dir("/sys/class/drm")
        .into_iter()
        .flatten()
        .flatten()
    {
        let name = e.file_name().to_string_lossy().into_owned();
        let dev = e.path().join("device");
        if !name.starts_with("card") || name.contains('-') || !dev.join("gpu_busy_percent").exists()
        {
            continue;
        }
        let n = |f: &str| read(dev.join(f)).and_then(|v| v.parse::<f64>().ok());
        let hw = std::fs::read_dir(dev.join("hwmon"))
            .ok()
            .and_then(|mut d| d.next())
            .and_then(|e| e.ok())
            .map(|e| e.path());
        let h = |f: &str| {
            hw.as_ref()
                .and_then(|p| read(p.join(f)))
                .and_then(|v| v.parse::<f64>().ok())
        };
        out.push(GpuSample {
            name: format!("AMD graphics ({name})"),
            util: n("gpu_busy_percent"),
            memory_used: n("mem_info_vram_used").map(|v| v as u64),
            memory_total: n("mem_info_vram_total").map(|v| v as u64),
            temperature: h("temp1_input").map(|t| t / 1000.0),
            power: h("power1_average").map(|p| p / 1e6),
            power_limit: h("power1_cap").map(|p| p / 1e6),
            core_mhz: None,
            memory_mhz: None,
            fan: None,
        });
    }
    out
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
