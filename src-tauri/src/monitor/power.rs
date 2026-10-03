use std::time::Instant;

use super::model::{Battery, Power};
use crate::common::sysfs::read;

const RAPL: &str = "/sys/class/powercap/intel-rapl:0";

pub(crate) struct Rapl {
    energy_uj: u64,
    at: Instant,
}

fn num(path: &str) -> Option<f64> {
    read(path).and_then(|v| v.parse::<f64>().ok())
}

pub(crate) fn watts_between(prev_uj: u64, now_uj: u64, wrap_uj: u64, seconds: f64) -> Option<f64> {
    if seconds <= 0.0 {
        return None;
    }
    let delta = if now_uj >= prev_uj {
        now_uj - prev_uj
    } else if wrap_uj > prev_uj {
        wrap_uj - prev_uj + now_uj
    } else {
        return None;
    };
    Some(delta as f64 / 1e6 / seconds)
}

fn cpu_watts(state: &mut Option<Rapl>) -> (Option<f64>, bool) {
    let Some(energy) = read(format!("{RAPL}/energy_uj")).and_then(|v| v.parse::<u64>().ok()) else {
        *state = None;
        return (None, false);
    };
    let now = Instant::now();
    let wrap = num(&format!("{RAPL}/max_energy_range_uj")).unwrap_or(0.0) as u64;
    let watts = state.as_ref().and_then(|p| {
        watts_between(
            p.energy_uj,
            energy,
            wrap,
            now.duration_since(p.at).as_secs_f64(),
        )
    });
    *state = Some(Rapl {
        energy_uj: energy,
        at: now,
    });
    (watts, true)
}

pub(crate) fn battery_estimates(
    status: &str,
    now_wh: Option<f64>,
    full_wh: Option<f64>,
    design_wh: Option<f64>,
    watts: Option<f64>,
) -> (Option<u64>, Option<f64>) {
    let seconds = watts.filter(|w| *w > 0.1).and_then(|w| match status {
        "Discharging" => now_wh.map(|n| n / w * 3600.0),
        "Charging" => Some((full_wh? - now_wh?).max(0.0) / w * 3600.0),
        _ => None,
    });
    let health = match (full_wh, design_wh) {
        (Some(f), Some(d)) if d > 0.0 => Some((f / d * 100.0).min(100.0)),
        _ => None,
    };
    (seconds.map(|s| s as u64), health)
}

fn batteries() -> (Vec<Battery>, Option<bool>) {
    let mut out = Vec::new();
    let mut ac = None;
    let mut dirs: Vec<_> = std::fs::read_dir("/sys/class/power_supply")
        .into_iter()
        .flatten()
        .flatten()
        .map(|e| e.path())
        .collect();
    dirs.sort();
    for dir in dirs {
        let p = |f: &str| dir.join(f).to_string_lossy().into_owned();
        let kind = read(p("type")).unwrap_or_default();
        if kind == "Mains" {
            if let Some(online) = num(&p("online")) {
                ac = Some(ac.unwrap_or(false) || online > 0.0);
            }
            continue;
        }
        if kind != "Battery" || read(p("scope")).as_deref() == Some("Device") {
            continue;
        }
        let voltage = num(&p("voltage_now")).map(|v| v / 1e6);
        let wh = |energy: &str, charge: &str| {
            num(&p(energy))
                .map(|v| v / 1e6)
                .or_else(|| Some(num(&p(charge))? / 1e6 * voltage?))
        };
        let (now_wh, full_wh, design_wh) = (
            wh("energy_now", "charge_now"),
            wh("energy_full", "charge_full"),
            wh("energy_full_design", "charge_full_design"),
        );
        let watts = num(&p("power_now"))
            .map(|w| w / 1e6)
            .or_else(|| Some(num(&p("current_now"))? / 1e6 * voltage?));
        let status = read(p("status")).unwrap_or_else(|| "Unknown".into());
        let (seconds_left, health) = battery_estimates(&status, now_wh, full_wh, design_wh, watts);
        let percent = num(&p("capacity")).or_else(|| Some(now_wh? / full_wh? * 100.0));
        let Some(percent) = percent else { continue };
        out.push(Battery {
            name: dir
                .file_name()
                .map(|n| n.to_string_lossy().into_owned())
                .unwrap_or_default(),
            percent,
            status,
            watts,
            seconds_left,
            health,
            cycles: num(&p("cycle_count")).map(|c| c as u64).filter(|c| *c > 0),
        });
    }
    (out, ac)
}

pub(crate) fn power(state: &mut Option<Rapl>) -> Power {
    let (watts, readable) = cpu_watts(state);
    let (batteries, ac_online) = batteries();
    let limit = |n: u8| num(&format!("{RAPL}/constraint_{n}_power_limit_uw")).map(|u| u / 1e6);
    Power {
        cpu_watts: watts,
        cpu_readable: readable,
        cpu_limit_sustained: limit(0),
        cpu_limit_boost: limit(1),
        ac_online,
        batteries,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn watts_are_energy_over_time() {
        assert_eq!(watts_between(1_000_000, 16_000_000, 0, 1.0), Some(15.0));
        assert_eq!(watts_between(0, 5_000_000, 0, 0.5), Some(10.0));
    }

    #[test]
    fn a_wrapped_counter_still_gives_a_reading() {
        assert_eq!(
            watts_between(990_000_000, 4_000_000, 1_000_000_000, 1.0),
            Some(14.0)
        );
        assert_eq!(watts_between(5, 1, 0, 1.0), None);
        assert_eq!(watts_between(1, 2, 0, 0.0), None);
    }

    #[test]
    fn battery_time_and_health_follow_the_status() {
        let (secs, health) = battery_estimates(
            "Discharging",
            Some(30.0),
            Some(50.0),
            Some(60.0),
            Some(15.0),
        );
        assert_eq!(secs, Some(7200));
        assert!((health.unwrap() - 83.333).abs() < 0.01);
        let (secs, _) =
            battery_estimates("Charging", Some(30.0), Some(50.0), Some(60.0), Some(20.0));
        assert_eq!(secs, Some(3600));
        let (secs, _) = battery_estimates("Full", Some(50.0), Some(50.0), None, Some(5.0));
        assert_eq!(secs, None);
        let (secs, health) = battery_estimates("Discharging", Some(30.0), None, None, Some(0.0));
        assert_eq!((secs, health), (None, None));
    }
}
