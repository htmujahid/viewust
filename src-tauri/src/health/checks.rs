use super::model::{Check, Severity};

pub(crate) fn check(
    severity: Severity,
    title: impl Into<String>,
    detail: impl Into<String>,
    link: Option<&'static str>,
) -> Check {
    Check {
        severity,
        title: title.into(),
        detail: detail.into(),
        link,
    }
}

pub(crate) fn failed_services(count: usize) -> Check {
    match count {
        0 => check(
            Severity::Ok,
            "Services",
            "No service has failed",
            Some("/services"),
        ),
        n => check(
            Severity::Danger,
            "Services",
            format!(
                "{n} service{} failed",
                if n == 1 { " has" } else { "s have" }
            ),
            Some("/services"),
        ),
    }
}

/// A healthy Linux still logs a few errors; only a pile of them means something.
pub(crate) fn journal_errors(count: usize, truncated: bool) -> Check {
    let severity = if truncated || count >= 100 {
        Severity::Danger
    } else if count >= 10 {
        Severity::Warn
    } else {
        Severity::Ok
    };
    let detail = match (count, truncated) {
        (0, _) => "No errors in the journal this boot".to_owned(),
        (n, false) => format!(
            "{n} error{} in the journal this boot",
            if n == 1 { "" } else { "s" }
        ),
        (n, true) => format!("Over {n} errors in the journal this boot"),
    };
    check(severity, "System log", detail, Some("/os/logs"))
}

/// `df --output=pcent,fstype,target` rows, already split. Only real filesystems count:
/// efivars and squashfs images sit at odd percentages by nature.
const REAL_FS: &[&str] = &[
    "ext2", "ext3", "ext4", "xfs", "btrfs", "zfs", "f2fs", "jfs", "ntfs", "ntfs3", "fuseblk",
    "vfat", "exfat", "bcachefs",
];

pub(crate) fn parse_df_percent(text: &str) -> Vec<(u8, String, String)> {
    text.lines()
        .skip(1)
        .filter_map(|line| {
            let mut f = line.split_whitespace();
            let pcent: u8 = f.next()?.strip_suffix('%')?.parse().ok()?;
            let fstype = f.next()?.to_owned();
            let target = f.collect::<Vec<_>>().join(" ");
            (!target.is_empty()).then_some((pcent, fstype, target))
        })
        .collect()
}

pub(crate) fn full_filesystems(rows: &[(u8, String, String)]) -> Check {
    let real: Vec<&(u8, String, String)> = rows
        .iter()
        .filter(|(_, fstype, _)| REAL_FS.contains(&fstype.as_str()))
        .collect();
    let worst = real.iter().max_by_key(|(p, _, _)| p);
    match worst {
        Some((p, _, mount)) if *p >= 95 => check(
            Severity::Danger,
            "Disk space",
            format!("{mount} is {p}% full"),
            Some("/os/filesystems"),
        ),
        Some((p, _, mount)) if *p >= 85 => check(
            Severity::Warn,
            "Disk space",
            format!("{mount} is {p}% full"),
            Some("/os/filesystems"),
        ),
        Some((p, _, mount)) => check(
            Severity::Ok,
            "Disk space",
            format!("Fullest filesystem is {mount} at {p}%"),
            Some("/os/filesystems"),
        ),
        None => check(
            Severity::Ok,
            "Disk space",
            "No filesystems to measure",
            None,
        ),
    }
}

pub(crate) fn memory(total: u64, available: u64, swap_total: u64, swap_free: u64) -> Check {
    if total == 0 {
        return check(
            Severity::Ok,
            "Memory",
            "Couldn't be read",
            Some("/os/memory"),
        );
    }
    let avail_pct = available * 100 / total;
    let swap_used_pct = (swap_total.saturating_sub(swap_free) * 100)
        .checked_div(swap_total)
        .unwrap_or(0);
    if avail_pct < 5 {
        check(
            Severity::Danger,
            "Memory",
            format!("Only {avail_pct}% of memory is still available"),
            Some("/os/memory"),
        )
    } else if avail_pct < 12 || swap_used_pct > 50 {
        check(
            Severity::Warn,
            "Memory",
            if avail_pct < 12 {
                format!("{avail_pct}% of memory left")
            } else {
                format!("Swap is {swap_used_pct}% used")
            },
            Some("/os/memory"),
        )
    } else {
        check(
            Severity::Ok,
            "Memory",
            format!("{avail_pct}% available · swap {swap_used_pct}% used"),
            Some("/os/memory"),
        )
    }
}

/// One line of `/proc/pressure/*`: `some avg10=1.23 avg60=0.50 …` → the avg60 figure.
pub(crate) fn parse_pressure(text: &str, which: &str) -> Option<f64> {
    text.lines()
        .find(|l| l.starts_with(which))?
        .split_whitespace()
        .find_map(|w| w.strip_prefix("avg60="))
        .and_then(|v| v.parse().ok())
}

pub(crate) fn pressure(name: &'static str, some_avg60: Option<f64>, link: &'static str) -> Check {
    match some_avg60 {
        Some(v) if v >= 40.0 => check(
            Severity::Danger,
            name,
            format!("Programs waited for it {v:.0}% of the last minute"),
            Some(link),
        ),
        Some(v) if v >= 15.0 => check(
            Severity::Warn,
            name,
            format!("Programs waited for it {v:.0}% of the last minute"),
            Some(link),
        ),
        Some(v) => check(
            Severity::Ok,
            name,
            format!("No real waiting ({v:.1}%)"),
            Some(link),
        ),
        None => check(
            Severity::Ok,
            name,
            "Pressure figures aren't available",
            None,
        ),
    }
}

pub(crate) fn temperature(sensors: &[(String, f64, Option<f64>)]) -> Check {
    // Judge each sensor against its own limit when it declares one; 90 °C otherwise.
    let hot = |t: f64, limit: Option<f64>| t >= limit.unwrap_or(90.0);
    let near = |t: f64, limit: Option<f64>| t >= limit.unwrap_or(90.0) * 0.9;
    if let Some((name, t, _)) = sensors.iter().find(|(_, t, l)| hot(*t, *l)) {
        return check(
            Severity::Danger,
            "Temperatures",
            format!("{name} is at {t:.0} °C"),
            Some("/monitor"),
        );
    }
    if let Some((name, t, _)) = sensors.iter().find(|(_, t, l)| near(*t, *l)) {
        return check(
            Severity::Warn,
            "Temperatures",
            format!("{name} is at {t:.0} °C, close to its limit"),
            Some("/monitor"),
        );
    }
    match sensors.iter().max_by(|a, b| a.1.total_cmp(&b.1)) {
        Some((name, t, _)) => check(
            Severity::Ok,
            "Temperatures",
            format!("Hottest sensor is {name} at {t:.0} °C"),
            Some("/monitor"),
        ),
        None => check(Severity::Ok, "Temperatures", "No sensors found", None),
    }
}

/// Taint bit 7 means the kernel died once; bit 9 that it hit a warning. The proprietary-module
/// bits are everyday life and don't make a machine unhealthy.
pub(crate) fn kernel_taint(mask: u64) -> Check {
    if mask & (1 << 7) != 0 {
        check(
            Severity::Danger,
            "Kernel",
            "The kernel has reported a fatal error since boot",
            Some("/os/kernel"),
        )
    } else if mask & (1 << 9) != 0 {
        check(
            Severity::Warn,
            "Kernel",
            "The kernel has hit an internal warning since boot",
            Some("/os/kernel"),
        )
    } else {
        check(
            Severity::Ok,
            "Kernel",
            "No kernel errors or warnings",
            Some("/os/kernel"),
        )
    }
}

fn kernel_numbers(name: &str) -> Vec<u64> {
    name.split(|c: char| !c.is_ascii_digit())
        .filter(|p| !p.is_empty())
        .filter_map(|p| p.parse().ok())
        .collect()
}

/// Running `7.0.0-34-generic` while `/boot` holds a newer image means a restart is pending.
pub(crate) fn pending_kernel(running: &str, installed: &[String]) -> Check {
    let newest = installed
        .iter()
        .filter_map(|p| p.rsplit('/').next()?.strip_prefix("vmlinuz-"))
        .max_by_key(|v| kernel_numbers(v));
    match newest {
        Some(newest) if kernel_numbers(newest) > kernel_numbers(running) => check(
            Severity::Warn,
            "Updates",
            format!("Kernel {newest} is installed but {running} is running; restart to use it"),
            Some("/os/boot"),
        ),
        _ => check(
            Severity::Ok,
            "Updates",
            "The newest installed kernel is running",
            None,
        ),
    }
}

pub(crate) fn battery(full: Option<(u64, u64)>) -> Option<Check> {
    let (now_full, design) = full?;
    if design == 0 {
        return None;
    }
    let health = now_full * 100 / design;
    Some(if health < 60 {
        check(
            Severity::Warn,
            "Battery",
            format!("The battery holds {health}% of what it was built for"),
            None,
        )
    } else {
        check(
            Severity::Ok,
            "Battery",
            format!("Holds {health}% of its design capacity"),
            None,
        )
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn services_and_logs_scale_with_their_counts() {
        assert_eq!(failed_services(0).severity, Severity::Ok);
        assert_eq!(failed_services(2).severity, Severity::Danger);
        assert_eq!(journal_errors(3, false).severity, Severity::Ok);
        assert_eq!(journal_errors(30, false).severity, Severity::Warn);
        assert_eq!(journal_errors(200, true).severity, Severity::Danger);
    }

    #[test]
    fn only_real_filesystems_can_raise_the_disk_alarm() {
        let rows = parse_df_percent(
            "Use% Type Mounted on\n 52% efivarfs /sys/firmware/efi/efivars\n100% squashfs /snap/x/1\n 96% ext4 /home\n 49% ext4 /\n",
        );
        let c = full_filesystems(&rows);
        assert_eq!(c.severity, Severity::Danger);
        assert!(c.detail.contains("/home"));
        let calm = full_filesystems(&parse_df_percent(
            "Use% Type On\n 52% efivarfs /e\n 49% ext4 /\n",
        ));
        assert_eq!(calm.severity, Severity::Ok);
    }

    #[test]
    fn memory_warns_when_little_is_left_or_swap_is_deep() {
        assert_eq!(memory(100, 50, 100, 100).severity, Severity::Ok);
        assert_eq!(memory(100, 8, 0, 0).severity, Severity::Warn);
        assert_eq!(memory(100, 3, 0, 0).severity, Severity::Danger);
        assert_eq!(memory(100, 50, 100, 40).severity, Severity::Warn);
    }

    #[test]
    fn pressure_reads_the_minute_average() {
        let text = "some avg10=0.00 avg60=22.51 avg300=8.00 total=1\nfull avg10=0.00 avg60=1.00 avg300=0.00 total=1\n";
        assert_eq!(parse_pressure(text, "some"), Some(22.51));
        assert_eq!(parse_pressure(text, "full"), Some(1.0));
        assert_eq!(
            pressure("Processor", Some(22.51), "/monitor").severity,
            Severity::Warn
        );
        assert_eq!(
            pressure("Processor", Some(2.0), "/monitor").severity,
            Severity::Ok
        );
        assert_eq!(
            pressure("Processor", None, "/monitor").severity,
            Severity::Ok
        );
    }

    #[test]
    fn sensors_are_judged_against_their_own_limits() {
        let sensors = vec![
            ("CPU".to_owned(), 55.0, Some(95.0)),
            ("NVMe".to_owned(), 68.0, Some(70.0)),
        ];
        assert_eq!(temperature(&sensors).severity, Severity::Warn);
        let hot = vec![("GPU".to_owned(), 91.0, None)];
        assert_eq!(temperature(&hot).severity, Severity::Danger);
        let fine = vec![("CPU".to_owned(), 45.0, Some(95.0))];
        let c = temperature(&fine);
        assert_eq!(c.severity, Severity::Ok);
        assert!(c.detail.contains("45"));
    }

    #[test]
    fn taint_cares_about_crashes_not_proprietary_drivers() {
        assert_eq!(kernel_taint(0).severity, Severity::Ok);
        assert_eq!(
            kernel_taint(4097).severity,
            Severity::Ok,
            "nvidia + out-of-tree"
        );
        assert_eq!(kernel_taint(1 << 9).severity, Severity::Warn);
        assert_eq!(kernel_taint(1 << 7).severity, Severity::Danger);
    }

    #[test]
    fn a_newer_installed_kernel_asks_for_a_restart() {
        let installed = vec![
            "/boot/vmlinuz-7.0.0-31-generic".to_owned(),
            "/boot/vmlinuz-7.0.0-34-generic".to_owned(),
        ];
        assert_eq!(
            pending_kernel("7.0.0-34-generic", &installed).severity,
            Severity::Ok
        );
        let c = pending_kernel("7.0.0-31-generic", &installed);
        assert_eq!(c.severity, Severity::Warn);
        assert!(c.detail.contains("7.0.0-34"));
        assert_eq!(pending_kernel("8.0.0-1", &[]).severity, Severity::Ok);
    }

    #[test]
    fn battery_health_is_full_against_design() {
        assert_eq!(battery(Some((55, 100))).unwrap().severity, Severity::Warn);
        assert_eq!(battery(Some((88, 100))).unwrap().severity, Severity::Ok);
        assert!(battery(None).is_none());
        assert!(battery(Some((50, 0))).is_none());
    }
}
