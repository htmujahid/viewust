use std::collections::HashMap;
use std::path::Path;

use sysinfo::System;

use super::model::OsSummary;
use super::{modules, packages, release};
use crate::common::cmd::{run, run_any};
use crate::common::sysfs::read;
use crate::common::Details;

fn first_line(text: Option<String>) -> Option<String> {
    text.and_then(|t| {
        t.lines()
            .map(str::trim)
            .find(|l| !l.is_empty())
            .map(str::to_owned)
    })
}

/// `key=value` lines from `timedatectl show`, `systemctl show` and friends.
fn properties(text: Option<String>) -> HashMap<String, String> {
    text.map(|t| crate::services::parse_properties(&t))
        .unwrap_or_default()
}

pub(crate) fn aslr(value: &str) -> String {
    match value.trim() {
        "2" => "Full (2)".into(),
        "1" => "Partial (1)".into(),
        "0" => "Off (0)".into(),
        other => other.to_owned(),
    }
}

/// The kernel's taint mask: 0 means nothing unusual has been loaded.
pub(crate) fn tainted(value: &str) -> String {
    match value.trim().parse::<u64>() {
        Ok(0) => "No".into(),
        Ok(n) => format!("Yes (flags {n})"),
        Err(_) => value.trim().to_owned(),
    }
}

/// `/sys/kernel/security/lockdown` reads `none [integrity] confidentiality`; the bracket is active.
pub(crate) fn lockdown(text: &str) -> Option<String> {
    let start = text.find('[')? + 1;
    let end = text.find(']')?;
    (end > start).then(|| text[start..end].to_owned())
}

/// `systemd-analyze time` begins "Startup finished in 5s (firmware) + … = 23s".
pub(crate) fn startup_time(text: &str) -> Option<String> {
    let line = text.lines().next()?;
    line.strip_prefix("Startup finished in ")
        .map(|s| s.trim().to_owned())
}

pub(crate) fn secure_boot(bytes: &[u8]) -> Option<&'static str> {
    // 4 bytes of attributes, then the value
    match bytes.get(4)? {
        1 => Some("Enabled"),
        0 => Some("Disabled"),
        _ => None,
    }
}

fn count(text: Option<String>, skip: usize) -> Option<usize> {
    text.map(|t| {
        t.lines()
            .filter(|l| !l.trim().is_empty())
            .count()
            .saturating_sub(skip)
    })
}

pub(crate) fn summary() -> OsSummary {
    // The slow lookups start first and run alongside the quick file reads.
    let (analyze, package_count, snaps, flatpaks) = std::thread::scope(|s| {
        let analyze = s.spawn(|| run_any("systemd-analyze", &["time"]));
        let pkgs = s.spawn(|| {
            let manager = packages::detect()?;
            let list = packages::list();
            Some((manager.label(), list.packages.len()))
        });
        let snaps = s.spawn(|| count(run("snap", &["list"]), 1));
        let flat = s.spawn(|| {
            count(
                run("flatpak", &["list", "--app", "--columns=application"]),
                0,
            )
        });
        (
            analyze.join().ok().flatten(),
            pkgs.join().ok().flatten(),
            snaps.join().ok().flatten(),
            flat.join().ok().flatten(),
        )
    });

    let os = release::os_release();
    let get = |k: &str| os.get(k).cloned().filter(|v| !v.is_empty());
    let name = get("PRETTY_NAME")
        .or_else(|| get("NAME"))
        .unwrap_or_else(|| "Linux".to_owned());
    let kernel = read("/proc/sys/kernel/osrelease").unwrap_or_default();
    let hostname = read("/proc/sys/kernel/hostname").unwrap_or_default();
    let architecture = std::env::consts::ARCH.to_owned();
    let env = |k: &str| std::env::var(k).ok().filter(|v| !v.is_empty());

    let mut d = Details::new();

    d.add("Operating system", "Name", &name);
    d.add_opt("Operating system", "Version", get("VERSION"));
    d.add_opt("Operating system", "Codename", get("VERSION_CODENAME"));
    d.add_opt("Operating system", "Distribution", get("ID"));
    d.add_opt("Operating system", "Based on", get("ID_LIKE"));
    d.add_opt("Operating system", "Build", get("BUILD_ID"));
    d.add_opt("Operating system", "Variant", get("VARIANT"));
    d.add_opt("Operating system", "Support ends", get("SUPPORT_END"));
    d.add_opt("Operating system", "Website", get("HOME_URL"));
    d.add_opt("Operating system", "Support", get("SUPPORT_URL"));
    d.add_opt("Operating system", "Bug reports", get("BUG_REPORT_URL"));
    d.add("Operating system", "Host name", &hostname);
    d.add("Operating system", "Architecture", &architecture);

    d.add("Kernel", "Release", &kernel);
    d.add_opt("Kernel", "Build", read("/proc/sys/kernel/version"));
    d.add_opt("Kernel", "Command line", read("/proc/cmdline"));
    d.add_opt(
        "Kernel",
        "Loaded modules",
        std::fs::read_to_string("/proc/modules")
            .ok()
            .map(|t| modules::parse_proc_modules(&t).len().to_string()),
    );
    d.add_opt(
        "Kernel",
        "Tainted",
        read("/proc/sys/kernel/tainted").map(|v| tainted(&v)),
    );
    d.add_opt("Kernel", "Process limit", read("/proc/sys/kernel/pid_max"));

    let efi = Path::new("/sys/firmware/efi").exists();
    d.add("Boot", "Firmware", if efi { "UEFI" } else { "Legacy BIOS" });
    if efi {
        d.add_opt(
            "Boot",
            "Secure Boot",
            std::fs::read(
                "/sys/firmware/efi/efivars/SecureBoot-8be4df61-93ca-11d2-aa0d-00e098032b8c",
            )
            .ok()
            .and_then(|b| secure_boot(&b))
            .map(str::to_owned),
        );
    }
    d.add_opt("Boot", "Started", first_line(run("uptime", &["-s"])));
    d.add(
        "Boot",
        "Running for",
        crate::common::format::duration(System::uptime()),
    );
    d.add_opt(
        "Boot",
        "Startup took",
        analyze.as_deref().and_then(startup_time),
    );
    d.add_opt("Boot", "Init system", read("/proc/1/comm"));
    d.add_opt(
        "Boot",
        "Default target",
        first_line(run("systemctl", &["get-default"])),
    );
    d.add_opt("Boot", "Boot ID", read("/proc/sys/kernel/random/boot_id"));

    d.add_opt(
        "Session",
        "Current user",
        env("USER").or_else(|| env("LOGNAME")),
    );
    d.add_opt("Session", "Desktop", env("XDG_CURRENT_DESKTOP"));
    d.add_opt("Session", "Session type", env("XDG_SESSION_TYPE"));
    d.add_opt(
        "Session",
        "Display",
        env("WAYLAND_DISPLAY").or_else(|| env("DISPLAY")),
    );
    d.add_opt("Session", "Shell", env("SHELL"));
    d.add_opt(
        "Session",
        "Signed-in sessions",
        count(run("loginctl", &["list-sessions", "--no-legend"]), 0).map(|n| n.to_string()),
    );

    let tz = properties(run(
        "timedatectl",
        &[
            "show",
            "-p",
            "Timezone",
            "-p",
            "NTPSynchronized",
            "-p",
            "NTP",
            "-p",
            "LocalRTC",
        ],
    ));
    d.add_opt("Language and time", "Language", env("LANG"));
    d.add_opt("Language and time", "Language fallback", env("LANGUAGE"));
    d.add_opt(
        "Language and time",
        "Time zone",
        tz.get("Timezone")
            .cloned()
            .or_else(|| read("/etc/timezone")),
    );
    d.add_opt(
        "Language and time",
        "Local time",
        first_line(run("date", &["+%F %T %Z (UTC%:z)"])),
    );
    let yes = |v: Option<&String>| v.map(|v| if v == "yes" { "Yes" } else { "No" }.to_owned());
    d.add_opt(
        "Language and time",
        "Clock synchronised",
        yes(tz.get("NTPSynchronized")),
    );
    d.add_opt("Language and time", "Network time", yes(tz.get("NTP")));
    d.add_opt(
        "Language and time",
        "Hardware clock in local time",
        yes(tz.get("LocalRTC")),
    );

    d.add_opt(
        "Security",
        "AppArmor",
        read("/sys/module/apparmor/parameters/enabled")
            .map(|v| if v == "Y" { "Enabled" } else { "Disabled" }.to_owned()),
    );
    d.add_opt(
        "Security",
        "SELinux",
        read("/sys/fs/selinux/enforce")
            .map(|v| if v == "1" { "Enforcing" } else { "Permissive" }.to_owned()),
    );
    d.add_opt(
        "Security",
        "Kernel lockdown",
        read("/sys/kernel/security/lockdown").and_then(|t| lockdown(&t)),
    );
    d.add_opt(
        "Security",
        "Address randomisation",
        read("/proc/sys/kernel/randomize_va_space").map(|v| aslr(&v)),
    );
    d.add_opt(
        "Security",
        "Program tracing limit",
        read("/proc/sys/kernel/yama/ptrace_scope"),
    );

    let virt = first_line(run_any("systemd-detect-virt", &[]));
    d.add_opt(
        "Virtualization",
        "Runs on",
        virt.map(|v| {
            if v == "none" {
                "Physical machine".to_owned()
            } else {
                v
            }
        }),
    );
    d.add_opt(
        "Virtualization",
        "Container",
        first_line(run_any("systemd-detect-virt", &["--container"])).filter(|v| v != "none"),
    );

    if let Some((manager, n)) = package_count {
        d.add("Software", "Package manager", manager);
        d.add("Software", "Installed packages", n.to_string());
    }
    d.add_opt("Software", "Snap packages", snaps.map(|n| n.to_string()));
    d.add_opt("Software", "Flatpak apps", flatpaks.map(|n| n.to_string()));
    d.add_opt(
        "Software",
        "C library",
        first_line(run_any("ldd", &["--version"])),
    );
    d.add_opt(
        "Software",
        "systemd",
        first_line(run("systemctl", &["--version"])),
    );

    OsSummary {
        name,
        version: get("VERSION").or_else(|| get("VERSION_ID")),
        kernel,
        hostname,
        architecture,
        boot_time: System::boot_time(),
        details: d.finish(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_kernel_taint_mask_is_put_plainly() {
        assert_eq!(tainted("0"), "No");
        assert_eq!(tainted("12288\n"), "Yes (flags 12288)");
        assert_eq!(tainted("weird"), "weird");
    }

    #[test]
    fn address_randomisation_levels_are_named() {
        assert_eq!(aslr("2"), "Full (2)");
        assert_eq!(aslr("0\n"), "Off (0)");
    }

    #[test]
    fn the_active_lockdown_mode_is_the_bracketed_one() {
        assert_eq!(
            lockdown("none [integrity] confidentiality").as_deref(),
            Some("integrity")
        );
        assert_eq!(
            lockdown("[none] integrity confidentiality").as_deref(),
            Some("none")
        );
        assert_eq!(lockdown("garbage"), None);
    }

    #[test]
    fn the_startup_time_line_is_picked_out() {
        let t = "Startup finished in 5.1s (firmware) + 3s (loader) + 20s (userspace) = 28s\ngraphical.target reached after 20s";
        assert_eq!(
            startup_time(t).as_deref(),
            Some("5.1s (firmware) + 3s (loader) + 20s (userspace) = 28s")
        );
        assert_eq!(startup_time("Bootup is not yet finished"), None);
    }

    #[test]
    fn secure_boot_is_the_fifth_byte() {
        assert_eq!(secure_boot(&[7, 0, 0, 0, 1]), Some("Enabled"));
        assert_eq!(secure_boot(&[7, 0, 0, 0, 0]), Some("Disabled"));
        assert_eq!(secure_boot(&[7, 0]), None);
    }

    #[test]
    fn this_machine_describes_itself() {
        let s = summary();
        assert!(!s.kernel.is_empty() && !s.hostname.is_empty());
        let get = |label: &str| {
            s.details
                .iter()
                .find(|d| serde_json::to_value(d).unwrap()["label"] == label)
                .is_some()
        };
        assert!(get("Release") && get("Name") && get("Architecture"));
        assert!(s.boot_time > 1_000_000_000);
    }
}
