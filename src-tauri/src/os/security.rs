use std::path::Path;

use super::info::bracketed;
use super::model::OsSecurity;
use crate::common::cmd::run;
use crate::common::sysfs::read;
use crate::common::Details;

pub(crate) fn kptr(value: &str) -> String {
    match value.trim() {
        "0" => "Visible to everyone (0)".into(),
        "1" => "Hidden from ordinary users (1)".into(),
        "2" => "Hidden from everyone (2)".into(),
        other => other.to_owned(),
    }
}

pub(crate) fn ptrace(value: &str) -> String {
    match value.trim() {
        "0" => "Any program of the same user (0)".into(),
        "1" => "Only parents and chosen debuggers (1)".into(),
        "2" => "Administrators only (2)".into(),
        "3" => "Nobody (3)".into(),
        other => other.to_owned(),
    }
}

pub(crate) fn aslr(value: &str) -> String {
    match value.trim() {
        "2" => "Full (2)".into(),
        "1" => "Partial (1)".into(),
        "0" => "Off (0)".into(),
        other => other.to_owned(),
    }
}

/// The first firewall service that systemd reports as active.
pub(crate) fn firewall_of(active: impl Fn(&str) -> bool) -> Option<&'static str> {
    [
        ("ufw.service", "ufw"),
        ("firewalld.service", "firewalld"),
        ("nftables.service", "nftables"),
        ("iptables.service", "iptables"),
    ]
    .into_iter()
    .find(|(unit, _)| active(unit))
    .map(|(_, name)| name)
}

fn secure_boot() -> Option<&'static str> {
    let bytes =
        std::fs::read("/sys/firmware/efi/efivars/SecureBoot-8be4df61-93ca-11d2-aa0d-00e098032b8c")
            .ok()?;
    super::info::secure_boot(&bytes)
}

pub(crate) fn snapshot() -> OsSecurity {
    let apparmor = read("/sys/module/apparmor/parameters/enabled")
        .map(|v| if v == "Y" { "Enabled" } else { "Disabled" }.to_owned());
    let selinux = read("/sys/fs/selinux/enforce")
        .map(|v| if v == "1" { "Enforcing" } else { "Permissive" }.to_owned());
    let lockdown = read("/sys/kernel/security/lockdown").and_then(|t| bracketed(&t));
    let secure_boot = if Path::new("/sys/firmware/efi").exists() {
        Some(secure_boot().unwrap_or("Unknown").to_owned())
    } else {
        None
    };

    let mut d = Details::new();
    d.add_opt("Access control", "AppArmor", apparmor.clone());
    d.add_opt("Access control", "SELinux", selinux.clone());
    if apparmor.is_none() && selinux.is_none() {
        d.add(
            "Access control",
            "Mandatory access control",
            "None detected",
        );
    }
    d.add_opt(
        "Access control",
        "Program tracing",
        read("/proc/sys/kernel/yama/ptrace_scope").map(|v| ptrace(&v)),
    );

    d.add_opt("Kernel", "Lockdown", lockdown.clone());
    d.add_opt(
        "Kernel",
        "Address randomisation",
        read("/proc/sys/kernel/randomize_va_space").map(|v| aslr(&v)),
    );
    d.add_opt(
        "Kernel",
        "Kernel addresses",
        read("/proc/sys/kernel/kptr_restrict").map(|v| kptr(&v)),
    );
    d.add_opt(
        "Kernel",
        "Kernel log",
        read("/proc/sys/kernel/dmesg_restrict").map(|v| {
            if v.trim() == "1" {
                "Administrators only".to_owned()
            } else {
                "Everyone can read it".to_owned()
            }
        }),
    );
    d.add_opt(
        "Kernel",
        "Unprivileged BPF",
        read("/proc/sys/kernel/unprivileged_bpf_disabled").map(|v| match v.trim() {
            "0" => "Allowed".to_owned(),
            "1" => "Blocked".to_owned(),
            "2" => "Blocked until restart".to_owned(),
            other => other.to_owned(),
        }),
    );
    d.add_opt(
        "Kernel",
        "User namespaces",
        read("/proc/sys/kernel/unprivileged_userns_clone").map(|v| {
            if v.trim() == "1" {
                "Ordinary users may create them".to_owned()
            } else {
                "Administrators only".to_owned()
            }
        }),
    );

    d.add_opt("Boot", "Secure Boot", secure_boot.clone());

    let active = |unit: &str| run("systemctl", &["is-active", "--quiet", unit]).is_some();
    d.add(
        "Network",
        "Firewall",
        firewall_of(active)
            .map(|f| format!("{f} is running"))
            .unwrap_or_else(|| "No firewall service is running".into()),
    );

    OsSecurity {
        apparmor,
        selinux,
        lockdown,
        secure_boot,
        details: d.finish(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn restriction_levels_are_put_plainly() {
        assert_eq!(kptr("1"), "Hidden from ordinary users (1)");
        assert_eq!(ptrace("2\n"), "Administrators only (2)");
        assert_eq!(aslr("2"), "Full (2)");
        assert_eq!(kptr("x"), "x");
    }

    #[test]
    fn the_first_running_firewall_wins() {
        assert_eq!(firewall_of(|u| u == "firewalld.service"), Some("firewalld"));
        assert_eq!(firewall_of(|u| u == "ufw.service"), Some("ufw"));
        assert_eq!(firewall_of(|_| false), None);
    }

    #[test]
    fn this_machine_reports_its_protections() {
        let s = snapshot();
        assert!(!s.details.is_empty());
    }
}
