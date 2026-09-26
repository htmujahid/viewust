//! Small helpers for reading `/sys` and `/proc`.

pub fn read(path: impl AsRef<std::path::Path>) -> Option<String> {
    std::fs::read_to_string(path)
        .ok()
        .map(|s| s.trim().to_owned())
        .filter(|s| !s.is_empty())
}

/// Temperature (°C) from the first hwmon chip with one of these names.
pub(crate) fn hwmon_temp(chips: &[&str], label: Option<&str>) -> Option<f64> {
    for e in std::fs::read_dir("/sys/class/hwmon").ok()?.flatten() {
        let p = e.path();
        if !chips.contains(&read(p.join("name"))?.as_str()) {
            continue;
        }
        for i in 1..32 {
            let want = label.map_or(true, |l| {
                read(p.join(format!("temp{i}_label"))).as_deref() == Some(l)
            });
            if want {
                if let Some(v) =
                    read(p.join(format!("temp{i}_input"))).and_then(|v| v.parse::<f64>().ok())
                {
                    return Some(v / 1000.0);
                }
            }
        }
    }
    None
}

pub(crate) fn dmi(file: &str) -> Option<String> {
    read(format!("/sys/class/dmi/id/{file}")).filter(|v| {
        let l = v.to_lowercase();
        !matches!(
            l.as_str(),
            "default string" | "to be filled by o.e.m." | "not specified" | "none"
        )
    })
}
