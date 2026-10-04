use super::model::Component;
use crate::common::sysfs::read;
use crate::common::Details;

pub(crate) struct FanReading {
    pub(crate) chip: String,
    pub(crate) label: Option<String>,
    pub(crate) index: u32,
    pub(crate) rpm: u32,
}

/// What a fan should be called: its chip's label when it gives one, otherwise its number.
pub(crate) fn fan_name(fan: &FanReading) -> String {
    match &fan.label {
        Some(label) => label.clone(),
        None if fan.index == 1 => "CPU fan".to_owned(),
        None => format!("Fan {}", fan.index),
    }
}

pub(crate) fn components(fans: &[FanReading]) -> Vec<Component> {
    fans.iter()
        .map(|fan| {
            let mut d = Details::new();
            d.add("Fan", "Speed", format!("{} rpm", fan.rpm));
            d.add_opt("Fan", "Label", fan.label.clone());
            d.add("Fan", "Sensor chip", &fan.chip);
            d.add("Fan", "Sensor", format!("fan{}", fan.index));
            if fan.label.is_none() && fan.index == 1 {
                d.add(
                    "Fan",
                    "Named by convention",
                    "Sensor chips usually wire the processor fan first",
                );
            }
            Component {
                id: format!("sys:fan:{}:{}", fan.chip, fan.index),
                kind: "fan",
                name: fan_name(fan),
                subtitle: Some(format!("{} rpm", fan.rpm)),
                details: d.finish(),
            }
        })
        .collect()
}

/// Every spinning fan the kernel reports. A header reading 0 rpm is an empty plug, not a fan.
pub(crate) fn fans() -> Vec<FanReading> {
    let mut out = Vec::new();
    let Ok(entries) = std::fs::read_dir("/sys/class/hwmon") else {
        return out;
    };
    for e in entries.flatten() {
        let dir = e.path();
        let Some(chip) = read(dir.join("name")) else {
            continue;
        };
        for index in 1..=16u32 {
            let Some(rpm) =
                read(dir.join(format!("fan{index}_input"))).and_then(|v| v.parse::<u32>().ok())
            else {
                continue;
            };
            if rpm == 0 {
                continue;
            }
            out.push(FanReading {
                label: read(dir.join(format!("fan{index}_label"))),
                chip: chip.clone(),
                index,
                rpm,
            });
        }
    }
    out.sort_by_key(|f| (f.chip.clone(), f.index));
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fan(chip: &str, index: u32, rpm: u32, label: Option<&str>) -> FanReading {
        FanReading {
            chip: chip.into(),
            label: label.map(str::to_owned),
            index,
            rpm,
        }
    }

    #[test]
    fn a_labelled_fan_keeps_its_name_and_fan1_is_called_the_cpu_fan() {
        assert_eq!(
            fan_name(&fan("nct6683", 2, 900, Some("CPU Fan"))),
            "CPU Fan"
        );
        assert_eq!(fan_name(&fan("nct6683", 1, 1200, None)), "CPU fan");
        assert_eq!(fan_name(&fan("nct6683", 3, 650, None)), "Fan 3");
    }

    #[test]
    fn components_carry_speed_chip_and_an_honest_naming_note() {
        let c = components(&[fan("nct6683", 1, 1180, None)]);
        assert_eq!(c.len(), 1);
        assert_eq!(c[0].kind, "fan");
        assert_eq!(c[0].subtitle.as_deref(), Some("1180 rpm"));
        let json = serde_json::to_string(&c[0].details).unwrap();
        assert!(json.contains("1180 rpm") && json.contains("convention"));
    }

    #[test]
    fn this_machine_reports_only_real_readings() {
        for f in fans() {
            assert!(f.rpm > 0);
        }
    }
}
