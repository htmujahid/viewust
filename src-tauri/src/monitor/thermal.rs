use std::path::Path;

use super::model::{FanSensor, TempSensor, Thermal};
use crate::common::sysfs::read;

pub(crate) fn group_title(chip: &str, device_model: Option<&str>) -> String {
    let model = device_model.map(str::trim).filter(|m| !m.is_empty());
    match chip {
        "coretemp" => "CPU".into(),
        "k10temp" | "zenpower" => "CPU".into(),
        "acpitz" => "Motherboard".into(),
        "amdgpu" => "AMD graphics".into(),
        "nouveau" => "NVIDIA graphics".into(),
        "i915" | "xe" => "Intel graphics".into(),
        "nvme" => model.map_or("NVMe drive".into(), |m| format!("NVMe · {m}")),
        "drivetemp" => model.map_or("Drive".into(), |m| format!("Drive · {m}")),
        c if c.starts_with("iwlwifi") || c.starts_with("mt79") || c.starts_with("ath") => {
            "Wi-Fi adapter".into()
        }
        c if c.starts_with("r8169") || c.starts_with("igc") || c.starts_with("e1000") => {
            "Ethernet adapter".into()
        }
        c if c.starts_with("pch_") => "Chipset".into(),
        c => c.to_owned(),
    }
}

pub(crate) fn indices(names: &[String], prefix: &str, suffix: &str) -> Vec<u32> {
    let mut out: Vec<u32> = names
        .iter()
        .filter_map(|n| n.strip_prefix(prefix)?.strip_suffix(suffix)?.parse().ok())
        .collect();
    out.sort_unstable();
    out
}

fn milli(dir: &Path, file: String) -> Option<f64> {
    read(dir.join(file))
        .and_then(|v| v.parse::<f64>().ok())
        .map(|v| v / 1000.0)
        .filter(|v| *v > 0.0)
}

pub(crate) fn thermal() -> Thermal {
    let mut dirs: Vec<_> = std::fs::read_dir("/sys/class/hwmon")
        .into_iter()
        .flatten()
        .flatten()
        .map(|e| e.path())
        .collect();
    dirs.sort_by_key(|p| {
        p.file_name()
            .and_then(|n| {
                n.to_string_lossy()
                    .trim_start_matches("hwmon")
                    .parse::<u32>()
                    .ok()
            })
            .unwrap_or(u32::MAX)
    });

    let mut sensors = Vec::new();
    let mut fans = Vec::new();
    for dir in dirs {
        let Some(chip) = read(dir.join("name")) else {
            continue;
        };
        let hw = dir
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default();
        let model = read(dir.join("device/model"));
        let group = group_title(&chip, model.as_deref());
        let files: Vec<String> = std::fs::read_dir(&dir)
            .into_iter()
            .flatten()
            .flatten()
            .map(|e| e.file_name().to_string_lossy().into_owned())
            .collect();

        let temps = indices(&files, "temp", "_input");
        for i in &temps {
            let Some(celsius) = milli(&dir, format!("temp{i}_input")) else {
                continue;
            };
            let label = read(dir.join(format!("temp{i}_label"))).unwrap_or_else(|| {
                if temps.len() == 1 {
                    group.clone()
                } else {
                    format!("Sensor {i}")
                }
            });
            sensors.push(TempSensor {
                id: format!("{hw}:temp{i}"),
                group: group.clone(),
                label,
                celsius,
                high: milli(&dir, format!("temp{i}_max")),
                critical: milli(&dir, format!("temp{i}_crit")),
            });
        }

        for i in indices(&files, "fan", "_input") {
            let Some(rpm) =
                read(dir.join(format!("fan{i}_input"))).and_then(|v| v.parse::<f64>().ok())
            else {
                continue;
            };
            fans.push(FanSensor {
                id: format!("{hw}:fan{i}"),
                group: group.clone(),
                label: read(dir.join(format!("fan{i}_label")))
                    .unwrap_or_else(|| format!("Fan {i}")),
                rpm,
            });
        }
    }
    Thermal { sensors, fans }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chips_get_readable_group_names() {
        assert_eq!(group_title("coretemp", None), "CPU");
        assert_eq!(group_title("acpitz", None), "Motherboard");
        assert_eq!(
            group_title("nvme", Some(" Samsung 970 ")),
            "NVMe · Samsung 970"
        );
        assert_eq!(group_title("nvme", None), "NVMe drive");
        assert_eq!(group_title("iwlwifi_1", None), "Wi-Fi adapter");
        assert_eq!(group_title("mystery", None), "mystery");
    }

    #[test]
    fn sensor_numbers_are_read_from_file_names_in_order() {
        let names: Vec<String> = [
            "temp10_input",
            "temp2_input",
            "temp2_max",
            "name",
            "temp1_input",
        ]
        .iter()
        .map(|s| s.to_string())
        .collect();
        assert_eq!(indices(&names, "temp", "_input"), vec![1, 2, 10]);
        assert!(indices(&names, "fan", "_input").is_empty());
    }
}
