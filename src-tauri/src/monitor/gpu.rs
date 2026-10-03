use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::OnceLock;

use super::model::GpuSample;
use crate::common::cmd::run;
use crate::common::sysfs::read;
use crate::system::pci::{Pci, PciDevice};

fn pci() -> &'static Pci {
    static PCI: OnceLock<Pci> = OnceLock::new();
    PCI.get_or_init(Pci::load)
}

struct Card {
    slot: String,
    vendor: String,
    drm: PathBuf,
    device: PathBuf,
}

fn cards() -> Vec<Card> {
    let mut out: Vec<Card> = Vec::new();
    for e in std::fs::read_dir("/sys/class/drm")
        .into_iter()
        .flatten()
        .flatten()
    {
        let name = e.file_name().to_string_lossy().into_owned();
        let numbered = name
            .strip_prefix("card")
            .is_some_and(|n| !n.is_empty() && n.bytes().all(|b| b.is_ascii_digit()));
        if !numbered {
            continue;
        }
        let device = e.path().join("device");
        let Ok(real) = std::fs::canonicalize(&device) else {
            continue;
        };
        let Some(slot) = real.file_name().map(|n| n.to_string_lossy().into_owned()) else {
            continue;
        };
        if out.iter().any(|c| c.slot == slot) {
            continue;
        }
        let Some(vendor) = read(device.join("vendor")) else {
            continue;
        };
        out.push(Card {
            slot,
            vendor: vendor.trim_start_matches("0x").to_lowercase(),
            drm: e.path(),
            device,
        });
    }
    out.sort_by(|a, b| a.slot.cmp(&b.slot));
    out
}

fn blank(id: String, vendor: &'static str, kind: &'static str, name: String) -> GpuSample {
    GpuSample {
        id,
        vendor,
        kind,
        name,
        util: None,
        memory_used: None,
        memory_total: None,
        temperature: None,
        power: None,
        power_limit: None,
        core_mhz: None,
        memory_mhz: None,
        fan: None,
    }
}

fn model_name(vendor: &str, dev: Option<&PciDevice>) -> String {
    match dev.and_then(|d| d.device_name.clone()) {
        Some(n) => format!("{vendor} {n}"),
        None => format!("{vendor} graphics"),
    }
}

pub(crate) fn normalize_bus_id(id: &str) -> String {
    let id = id.trim().to_lowercase();
    match id.split_once(':') {
        Some((domain, rest)) if domain.len() > 4 => {
            format!("{}:{rest}", &domain[domain.len() - 4..])
        }
        _ => id,
    }
}

pub(crate) fn parse_nvidia(text: &str) -> Vec<GpuSample> {
    let num = |s: &str| s.trim().parse::<f64>().ok();
    text.lines()
        .filter(|l| !l.trim().is_empty())
        .map(|l| {
            let f: Vec<&str> = l.split(", ").map(str::trim).collect();
            let g = |i: usize| f.get(i).and_then(|v| num(v));
            let mut s = blank(
                normalize_bus_id(f.first().copied().unwrap_or("")),
                "nvidia",
                "discrete",
                f.get(1).copied().unwrap_or("NVIDIA graphics").to_owned(),
            );
            s.util = g(2);
            s.memory_used = g(3).map(|m| (m as u64) << 20);
            s.memory_total = g(4).map(|m| (m as u64) << 20);
            s.temperature = g(5);
            s.power = g(6);
            s.power_limit = g(7);
            s.core_mhz = g(8);
            s.memory_mhz = g(9);
            s.fan = g(10);
            s
        })
        .collect()
}

fn nvidia() -> HashMap<String, GpuSample> {
    run(
        "nvidia-smi",
        &[
            "--query-gpu=pci.bus_id,name,utilization.gpu,memory.used,memory.total,temperature.gpu,power.draw,power.limit,clocks.gr,clocks.mem,fan.speed",
            "--format=csv,noheader,nounits",
        ],
    )
    .map(|t| parse_nvidia(&t))
    .unwrap_or_default()
    .into_iter()
    .map(|s| (s.id.clone(), s))
    .collect()
}

pub(crate) fn parse_dpm_current(text: &str) -> Option<f64> {
    text.lines()
        .find(|l| l.trim_end().ends_with('*'))
        .and_then(|l| {
            let value = l.split(':').nth(1)?.trim();
            let digits: String = value.chars().take_while(|c| c.is_ascii_digit()).collect();
            digits.parse().ok()
        })
}

fn hwmon_dir(device: &std::path::Path) -> Option<PathBuf> {
    let mut dirs: Vec<PathBuf> = std::fs::read_dir(device.join("hwmon"))
        .ok()?
        .flatten()
        .map(|e| e.path())
        .collect();
    dirs.sort();
    dirs.into_iter().next()
}

fn amd(c: &Card) -> GpuSample {
    let dev = pci().at(&c.slot);
    let mut s = blank(c.slot.clone(), "amd", "unknown", model_name("AMD", dev));
    let n = |f: &str| read(c.device.join(f)).and_then(|v| v.parse::<f64>().ok());
    let hw = hwmon_dir(&c.device);
    let h = |f: &str| {
        hw.as_ref()
            .and_then(|p| read(p.join(f)))
            .and_then(|v| v.parse::<f64>().ok())
    };
    s.util = n("gpu_busy_percent");
    s.memory_used = n("mem_info_vram_used").map(|v| v as u64);
    s.memory_total = n("mem_info_vram_total").map(|v| v as u64);
    s.temperature = h("temp1_input").map(|t| t / 1000.0);
    s.power = h("power1_average")
        .or_else(|| h("power1_input"))
        .map(|p| p / 1e6);
    s.power_limit = h("power1_cap").map(|p| p / 1e6);
    s.core_mhz = read(c.device.join("pp_dpm_sclk")).and_then(|t| parse_dpm_current(&t));
    s.memory_mhz = read(c.device.join("pp_dpm_mclk")).and_then(|t| parse_dpm_current(&t));
    s.fan = h("pwm1").map(|v| v / h("pwm1_max").unwrap_or(255.0) * 100.0);
    s
}

pub(crate) fn intel_kind(slot: &str) -> &'static str {
    match slot.split(':').nth(1) {
        Some("00") => "integrated",
        Some(_) => "discrete",
        None => "unknown",
    }
}

fn intel(c: &Card) -> GpuSample {
    let dev = pci().at(&c.slot);
    let mut s = blank(
        c.slot.clone(),
        "intel",
        intel_kind(&c.slot),
        model_name("Intel", dev),
    );
    let mhz = |p: PathBuf| read(p).and_then(|v| v.parse::<f64>().ok());
    s.core_mhz = mhz(c.drm.join("gt_cur_freq_mhz"))
        .or_else(|| mhz(c.drm.join("gt/gt0/rps_cur_freq_mhz")))
        .or_else(|| mhz(c.device.join("tile0/gt0/freq0/cur_freq")));
    if let Some(hw) = hwmon_dir(&c.device) {
        let h = |f: &str| read(hw.join(f)).and_then(|v| v.parse::<f64>().ok());
        s.temperature = h("temp1_input").map(|t| t / 1000.0);
        s.power = h("power1_input").map(|p| p / 1e6);
    }
    s
}

pub(crate) fn gpus() -> Vec<GpuSample> {
    let mut smi = nvidia();
    let mut out = Vec::new();
    for c in cards() {
        match c.vendor.as_str() {
            "10de" => {
                let sample = smi.remove(&c.slot).unwrap_or_else(|| {
                    blank(
                        c.slot.clone(),
                        "nvidia",
                        "discrete",
                        model_name("NVIDIA", pci().at(&c.slot)),
                    )
                });
                out.push(sample);
            }
            "1002" => out.push(amd(&c)),
            "8086" => out.push(intel(&c)),
            _ => {}
        }
    }
    let mut rest: Vec<GpuSample> = smi.into_values().collect();
    rest.sort_by(|a, b| a.id.cmp(&b.id));
    out.extend(rest);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bus_ids_match_the_sysfs_form() {
        assert_eq!(normalize_bus_id("00000000:01:00.0"), "0000:01:00.0");
        assert_eq!(normalize_bus_id("0000:C1:00.0"), "0000:c1:00.0");
    }

    #[test]
    fn nvidia_rows_are_parsed_and_gaps_become_none() {
        let text = "00000000:01:00.0, NVIDIA GeForce GTX 1060 3GB, 9, 834, 3072, 55, 22.5, 120.00, 1582, 4006, 48\n\
                    00000000:02:00.0, NVIDIA T400, [N/A], 10, 2048, 40, [N/A], 31.50, 300, 5000, [N/A]\n";
        let gpus = parse_nvidia(text);
        assert_eq!(gpus.len(), 2);
        assert_eq!(gpus[0].id, "0000:01:00.0");
        assert_eq!(gpus[0].util, Some(9.0));
        assert_eq!(gpus[0].memory_total, Some(3072 << 20));
        assert_eq!(gpus[1].id, "0000:02:00.0");
        assert_eq!(gpus[1].util, None);
        assert_eq!(gpus[1].power, None);
        assert_eq!(gpus[1].fan, None);
        assert_eq!(gpus[1].power_limit, Some(31.5));
    }

    #[test]
    fn the_active_dpm_line_is_the_current_clock() {
        let text = "0: 500Mhz\n1: 1200Mhz *\n2: 2100Mhz\n";
        assert_eq!(parse_dpm_current(text), Some(1200.0));
        assert_eq!(parse_dpm_current("0: 500Mhz\n"), None);
    }

    #[test]
    fn intel_graphics_on_bus_zero_are_integrated() {
        assert_eq!(intel_kind("0000:00:02.0"), "integrated");
        assert_eq!(intel_kind("0000:03:00.0"), "discrete");
    }
}
