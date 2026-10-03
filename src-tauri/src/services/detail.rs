use std::process::Command;

use super::list::{counter, parse_properties};
use super::model::ServiceDetail;
use crate::common::cmd::run;
use crate::common::format::{duration, format_bytes};
use crate::common::Details;

const LOG_LINES: &str = "60";

pub(crate) fn valid_unit(unit: &str) -> bool {
    unit.ends_with(".service")
        && !unit.starts_with('-')
        && unit
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '-' | '_' | '@' | ':' | '\\'))
}

pub(crate) fn command_line(exec_start: &str) -> Option<String> {
    let rest = exec_start.split("argv[]=").nth(1)?;
    let line = rest.split(" ; ").next()?.trim();
    (!line.is_empty()).then(|| line.to_owned())
}

pub(crate) fn word_list(value: &str, limit: usize) -> Option<String> {
    let words: Vec<&str> = value.split_whitespace().collect();
    if words.is_empty() {
        return None;
    }
    let shown = words
        .iter()
        .take(limit)
        .copied()
        .collect::<Vec<_>>()
        .join(", ");
    Some(if words.len() > limit {
        format!("{shown} and {} more", words.len() - limit)
    } else {
        shown
    })
}

fn nonempty(v: Option<&String>) -> Option<String> {
    v.map(|s| s.trim().to_owned()).filter(|s| !s.is_empty())
}

fn logs(unit: &str) -> (Vec<String>, Option<String>) {
    let Ok(out) = Command::new("journalctl")
        .args(["-u", unit, "-n", LOG_LINES, "--no-pager", "-o", "short-iso"])
        .output()
    else {
        return (
            Vec::new(),
            Some("The system journal isn't available.".into()),
        );
    };
    let text = String::from_utf8_lossy(&out.stdout);
    let errors = String::from_utf8_lossy(&out.stderr);
    let lines: Vec<String> = text
        .lines()
        .filter(|l| !l.starts_with("-- "))
        .map(str::to_owned)
        .collect();
    if lines.is_empty() {
        let note = if errors.to_lowercase().contains("permission") {
            "Reading the journal needs membership of the systemd-journal group."
        } else {
            "No log entries for this service."
        };
        return (Vec::new(), Some(note.into()));
    }
    (lines, None)
}

pub(crate) fn detail(unit: &str) -> ServiceDetail {
    let missing = || ServiceDetail {
        unit: unit.to_owned(),
        found: false,
        details: Vec::new(),
        logs: Vec::new(),
        logs_note: None,
    };
    if !valid_unit(unit) {
        return missing();
    }
    let Some(text) = run("systemctl", &["show", unit, "--no-pager"]) else {
        return missing();
    };
    let p = parse_properties(&text);
    if p.get("LoadState").map(String::as_str) == Some("not-found") {
        return missing();
    }
    let get = |k: &str| nonempty(p.get(k));

    let mut d = Details::new();
    d.add_opt("Service", "Description", get("Description"));
    d.add_opt("Service", "Loaded", get("LoadState"));
    d.add_opt("Service", "Unit file", get("FragmentPath"));
    d.add_opt("Service", "Starts at boot", get("UnitFileState"));
    d.add_opt("Service", "Documentation", get("Documentation"));

    d.add(
        "State",
        "Status",
        format!(
            "{} ({})",
            get("ActiveState").unwrap_or_default(),
            get("SubState").unwrap_or_default()
        ),
    );
    d.add_opt("State", "Active since", get("ActiveEnterTimestamp"));
    d.add_opt("State", "Last stopped", get("InactiveEnterTimestamp"));
    d.add_opt("State", "Result", get("Result").filter(|r| r != "success"));
    d.add_opt(
        "State",
        "Exit status",
        get("ExecMainStatus").filter(|s| s != "0"),
    );
    d.add_opt("State", "Restarts", get("NRestarts").filter(|n| n != "0"));

    d.add_opt(
        "Process",
        "Main PID",
        counter(p.get("MainPID"))
            .filter(|p| *p > 0)
            .map(|p| p.to_string()),
    );
    d.add_opt(
        "Process",
        "Tasks",
        counter(p.get("TasksCurrent")).map(|t| match counter(p.get("TasksMax")) {
            Some(max) => format!("{t} of {max}"),
            None => t.to_string(),
        }),
    );
    d.add_opt(
        "Process",
        "Memory",
        counter(p.get("MemoryCurrent")).map(format_bytes),
    );
    d.add_opt(
        "Process",
        "Peak memory",
        counter(p.get("MemoryPeak")).map(format_bytes),
    );
    d.add_opt(
        "Process",
        "CPU time used",
        counter(p.get("CPUUsageNSec")).map(|n| duration(n / 1_000_000_000)),
    );

    d.add_opt(
        "Command",
        "Command line",
        get("ExecStart").and_then(|e| command_line(&e)),
    );
    d.add(
        "Run as",
        "User",
        get("User").unwrap_or_else(|| "root".into()),
    );
    d.add_opt("Run as", "Group", get("Group"));
    d.add_opt("Run as", "Restart policy", get("Restart"));

    d.add_opt(
        "Dependencies",
        "Needs",
        get("Requires").and_then(|v| word_list(&v, 8)),
    );
    d.add_opt(
        "Dependencies",
        "Wants",
        get("Wants").and_then(|v| word_list(&v, 8)),
    );
    d.add_opt(
        "Dependencies",
        "Starts after",
        get("After").and_then(|v| word_list(&v, 8)),
    );
    d.add_opt(
        "Dependencies",
        "Wanted by",
        get("WantedBy").and_then(|v| word_list(&v, 8)),
    );

    let (logs, logs_note) = logs(unit);
    ServiceDetail {
        unit: unit.to_owned(),
        found: true,
        details: d.finish(),
        logs,
        logs_note,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_plain_service_names_are_accepted() {
        assert!(valid_unit("ssh.service"));
        assert!(valid_unit("getty@tty1.service"));
        assert!(!valid_unit("--help"));
        assert!(!valid_unit("-x.service"));
        assert!(!valid_unit("ssh.socket"));
        assert!(!valid_unit("a b.service"));
        assert!(!valid_unit("a;rm.service"));
    }

    #[test]
    fn the_command_line_is_read_from_exec_start() {
        let raw = "{ path=/usr/sbin/sshd ; argv[]=/usr/sbin/sshd -D -o x ; ignore_errors=no ; start_time=[n/a] }";
        assert_eq!(command_line(raw), Some("/usr/sbin/sshd -D -o x".into()));
        assert_eq!(command_line(""), None);
        assert_eq!(command_line("{ path=/bin/x }"), None);
    }

    #[test]
    fn long_lists_are_shortened() {
        assert_eq!(word_list("a b", 8), Some("a, b".into()));
        assert_eq!(word_list("a b c d", 2), Some("a, b and 2 more".into()));
        assert_eq!(word_list("   ", 2), None);
    }
}
