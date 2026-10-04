use super::checks::*;
use super::model::{Check, HealthReport, Severity};
use crate::common::cmd::run;
use crate::common::sysfs::read;

fn hwmon_sensors() -> Vec<(String, f64, Option<f64>)> {
    let mut out = Vec::new();
    let Ok(entries) = std::fs::read_dir("/sys/class/hwmon") else {
        return out;
    };
    for e in entries.flatten() {
        let dir = e.path();
        let Some(chip) = read(dir.join("name")) else {
            continue;
        };
        for i in 1..16 {
            let Some(t) =
                read(dir.join(format!("temp{i}_input"))).and_then(|v| v.parse::<f64>().ok())
            else {
                continue;
            };
            let label = read(dir.join(format!("temp{i}_label")));
            let limit = [format!("temp{i}_crit"), format!("temp{i}_max")]
                .iter()
                .find_map(|f| read(dir.join(f)).and_then(|v| v.parse::<f64>().ok()))
                .map(|v| v / 1000.0)
                .filter(|v| *v > 40.0);
            let name = match label {
                Some(l) => format!("{chip} {l}"),
                None => chip.clone(),
            };
            out.push((name, t / 1000.0, limit));
        }
    }
    out
}

fn meminfo_value(text: &str, key: &str) -> u64 {
    text.lines()
        .find(|l| l.starts_with(key))
        .and_then(|l| l.split_whitespace().nth(1))
        .and_then(|v| v.parse().ok())
        .unwrap_or(0)
}

pub(crate) fn report() -> HealthReport {
    let mut checks: Vec<Check> = Vec::new();

    let failed = run(
        "systemctl",
        &["list-units", "--state=failed", "--no-legend", "--plain"],
    )
    .map(|t| t.lines().filter(|l| !l.trim().is_empty()).count())
    .unwrap_or(0);
    checks.push(failed_services(failed));

    let errors = run(
        "journalctl",
        &[
            "-b",
            "-p",
            "3",
            "-q",
            "--no-pager",
            "-n",
            "301",
            "-o",
            "cat",
        ],
    )
    .map(|t| t.lines().count());
    if let Some(n) = errors {
        checks.push(journal_errors(n.min(300), n > 300));
    }

    let df = run("df", &["-l", "--output=pcent,fstype,target"])
        .map(|t| parse_df_percent(&t))
        .unwrap_or_default();
    checks.push(full_filesystems(&df));

    if let Ok(mem) = std::fs::read_to_string("/proc/meminfo") {
        checks.push(memory(
            meminfo_value(&mem, "MemTotal"),
            meminfo_value(&mem, "MemAvailable"),
            meminfo_value(&mem, "SwapTotal"),
            meminfo_value(&mem, "SwapFree"),
        ));
    }

    let psi = |what: &str| {
        std::fs::read_to_string(format!("/proc/pressure/{what}"))
            .ok()
            .and_then(|t| parse_pressure(&t, "some"))
    };
    checks.push(pressure("Processor pressure", psi("cpu"), "/monitor/cpu"));
    checks.push(pressure(
        "Memory pressure",
        psi("memory"),
        "/monitor/memory",
    ));
    checks.push(pressure("Disk pressure", psi("io"), "/monitor"));

    checks.push(temperature(&hwmon_sensors()));

    checks.push(kernel_taint(
        read("/proc/sys/kernel/tainted")
            .and_then(|v| v.parse().ok())
            .unwrap_or(0),
    ));

    let installed: Vec<String> = std::fs::read_dir("/boot")
        .into_iter()
        .flatten()
        .flatten()
        .map(|e| e.path().to_string_lossy().into_owned())
        .filter(|p| p.contains("/vmlinuz-"))
        .collect();
    if let Some(running) = read("/proc/sys/kernel/osrelease") {
        checks.push(pending_kernel(&running, &installed));
    }

    for bat in std::fs::read_dir("/sys/class/power_supply")
        .into_iter()
        .flatten()
        .flatten()
    {
        let dir = bat.path();
        if read(dir.join("type")).as_deref() != Some("Battery") {
            continue;
        }
        let pair = |a: &str, b: &str| {
            Some((
                read(dir.join(a))?.parse().ok()?,
                read(dir.join(b))?.parse().ok()?,
            ))
        };
        let full = pair("energy_full", "energy_full_design")
            .or_else(|| pair("charge_full", "charge_full_design"));
        checks.extend(battery(full));
    }

    // The worst news first, then the reassurances.
    checks.sort_by(|a, b| {
        b.severity
            .partial_cmp(&a.severity)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    let count = |s: Severity| checks.iter().filter(|c| c.severity == s).count();
    HealthReport {
        problems: count(Severity::Danger),
        warnings: count(Severity::Warn),
        fine: count(Severity::Ok),
        checks,
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn this_machine_can_be_judged() {
        let r = super::report();
        assert!(r.checks.len() >= 8);
        assert_eq!(r.problems + r.warnings + r.fine, r.checks.len());
    }
}
