use std::path::Path;

use super::model::{VmRow, Vms};
use crate::common::cmd::run;

/// `virsh list --all` is a table: ` Id   Name   State`, dashes, then rows.
pub(crate) fn parse_virsh(text: &str) -> Vec<VmRow> {
    text.lines()
        .skip(2)
        .filter_map(|line| {
            let mut f = line.split_whitespace();
            let _id = f.next()?;
            let name = f.next()?.to_owned();
            let state = f.collect::<Vec<_>>().join(" ");
            (!state.is_empty()).then_some(VmRow {
                name,
                state,
                manager: "libvirt",
            })
        })
        .collect()
}

/// `machinectl list --no-legend`: `name class service os version addresses`.
pub(crate) fn parse_machinectl(text: &str) -> Vec<VmRow> {
    text.lines()
        .filter_map(|line| {
            let mut f = line.split_whitespace();
            let name = f.next()?.to_owned();
            let class = f.next().unwrap_or("machine").to_owned();
            Some(VmRow {
                name,
                state: format!("running ({class})"),
                manager: "systemd-machined",
            })
        })
        .collect()
}

fn installed(bin: &str) -> bool {
    ["/usr/bin", "/usr/local/bin", "/bin"]
        .iter()
        .any(|d| Path::new(&format!("{d}/{bin}")).exists())
}

pub(crate) fn snapshot() -> Vms {
    let mut managers = Vec::new();
    let mut vms = Vec::new();

    if installed("virsh") {
        managers.push("libvirt");
        // The system connection holds the real machines; it may need the libvirt group.
        let listed = run("virsh", &["-c", "qemu:///system", "list", "--all"])
            .or_else(|| run("virsh", &["list", "--all"]));
        if let Some(text) = listed {
            vms.extend(parse_virsh(&text));
        }
    }
    if installed("machinectl") {
        managers.push("systemd-machined");
        if let Some(text) = run("machinectl", &["list", "--no-legend"]) {
            vms.extend(parse_machinectl(&text));
        }
    }

    let note = if managers.is_empty() {
        Some("No virtual-machine manager (libvirt or systemd-machined) is installed.".to_owned())
    } else if vms.is_empty() {
        Some("No virtual machines are defined.".to_owned())
    } else {
        None
    };
    Vms {
        running: vms
            .iter()
            .filter(|v| v.state.starts_with("running"))
            .count(),
        managers,
        vms,
        note,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn virsh_tables_turn_into_rows() {
        let text = " Id   Name     State\n--------------------------\n 1    work     running\n -    old-vm   shut off\n";
        let v = parse_virsh(text);
        assert_eq!(v.len(), 2);
        assert_eq!(
            (v[0].name.as_str(), v[0].state.as_str()),
            ("work", "running")
        );
        assert_eq!(v[1].state, "shut off");
        assert!(parse_virsh("").is_empty());
    }

    #[test]
    fn machinectl_rows_carry_their_class() {
        let v = parse_machinectl("fedora-ws container systemd-nspawn fedora 40 -\n");
        assert_eq!(v.len(), 1);
        assert_eq!(v[0].state, "running (container)");
        assert_eq!(v[0].manager, "systemd-machined");
    }

    #[test]
    fn this_machine_answers_honestly_about_vms() {
        let v = snapshot();
        assert!(!v.managers.is_empty() || v.note.is_some());
    }
}
