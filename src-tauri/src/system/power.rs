//! The power supply, and what is known about power.

use super::model::*;
use crate::common::nvidia::*;
use crate::common::sysfs::*;
use crate::common::Details;

pub(crate) fn power() -> Component {
    let rapl = |n: u8| {
        read(format!(
            "/sys/class/powercap/intel-rapl:0/constraint_{n}_power_limit_uw"
        ))
        .and_then(|v| v.parse::<u64>().ok())
        .map(|u| u as f64 / 1e6)
    };
    let gpu_limits: f64 = nvidia_smi()
        .iter()
        .filter_map(|s| s.values.get(6)?.parse::<f64>().ok())
        .sum();
    let gpu_draw: f64 = nvidia_smi()
        .iter()
        .filter_map(|s| s.values.get(5)?.parse::<f64>().ok())
        .sum();

    let mut d = Details::new();
    d.add(
        "Power supply",
        "Model and wattage",
        "Not reported. A power supply doesn't talk to the computer, so no software can read its \
         model, rating or health. Check the label on the unit itself.",
    );
    d.add_opt(
        "What the computer does report",
        "CPU sustained limit",
        rapl(0).map(|w| format!("{w:.0} W")),
    );
    d.add_opt(
        "What the computer does report",
        "CPU boost limit",
        rapl(1).map(|w| format!("{w:.0} W")),
    );
    if gpu_limits > 0.0 {
        d.add(
            "What the computer does report",
            "Graphics card draw now",
            format!("{gpu_draw:.0} W"),
        );
        d.add(
            "What the computer does report",
            "Graphics card limit",
            format!("{gpu_limits:.0} W"),
        );
    }
    if gpu_limits > 0.0 || rapl(1).is_some() {
        let peak = rapl(1).or(rapl(0)).unwrap_or(0.0) + gpu_limits;
        d.add(
            "What the computer does report",
            "CPU + graphics peak limits",
            format!("about {peak:.0} W (drives, fans and the board add more)"),
        );
    }
    Component {
        id: "sys:psu".into(),
        kind: "psu",
        name: "Power supply".into(),
        subtitle: Some("Not reported by hardware".into()),
        details: d.finish(),
    }
}
