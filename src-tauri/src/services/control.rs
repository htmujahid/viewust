use crate::common::elevate;
use crate::error::{AppError, Result};

pub(crate) fn verb(action: &str) -> Option<&'static str> {
    Some(match action {
        "start" => "start",
        "stop" => "stop",
        "restart" => "restart",
        "reload" => "reload",
        "enable" => "enable",
        "disable" => "disable",
        _ => return None,
    })
}

/// A unit name goes to a program running as administrator, so only plain unit names get through.
pub(crate) fn valid_unit(unit: &str) -> bool {
    unit.len() <= 256
        && unit.ends_with(".service")
        && !unit.starts_with('-')
        && unit
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || "-_.@:\\".contains(c))
}

pub(crate) fn run(unit: &str, action: &str) -> Result<()> {
    let verb = verb(action).ok_or_else(|| AppError::Other(format!("Unknown action “{action}”")))?;
    if !valid_unit(unit) {
        return Err(AppError::Other(format!("“{unit}” is not a service name")));
    }
    elevate::run(
        &["/usr/bin/systemctl", "/bin/systemctl"],
        "systemctl",
        &[verb, "--", unit],
    )
    .map(drop)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_known_actions_pass() {
        for a in ["start", "stop", "restart", "reload", "enable", "disable"] {
            assert_eq!(verb(a), Some(a));
        }
        assert_eq!(verb("mask"), None);
        assert_eq!(verb("poweroff"), None);
    }

    #[test]
    fn only_plain_service_names_pass() {
        assert!(valid_unit("ssh.service"));
        assert!(valid_unit("getty@tty1.service"));
        assert!(valid_unit("systemd-fsck@dev-disk-by\\x2duuid.service"));
        assert!(!valid_unit("ssh"));
        assert!(!valid_unit("--now.service"));
        assert!(!valid_unit("a b.service"));
        assert!(!valid_unit("a;b.service"));
        assert!(!valid_unit("../x.service"));
    }
}
