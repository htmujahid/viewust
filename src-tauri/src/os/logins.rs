use std::collections::HashMap;

use super::model::{LoginSession, OsLogins};
use crate::common::cmd::run;
use crate::services::parse_properties;

/// `loginctl list-sessions --no-legend`: the first word of each line is the session id.
pub(crate) fn parse_session_ids(text: &str) -> Vec<String> {
    text.lines()
        .filter_map(|l| l.split_whitespace().next())
        .map(str::to_owned)
        .collect()
}

/// "Mon 2026-10-04 05:37:12 PKT" with the weekday dropped; already readable as is.
fn tidy_timestamp(value: &str) -> String {
    let mut words = value.split_whitespace();
    match words.next() {
        Some(day) if day.len() == 3 && day.chars().all(char::is_alphabetic) => {
            words.collect::<Vec<_>>().join(" ")
        }
        _ => value.to_owned(),
    }
}

pub(crate) fn session_from(props: &HashMap<String, String>, id: &str) -> LoginSession {
    let get = |k: &str| props.get(k).cloned().filter(|v| !v.is_empty());
    LoginSession {
        id: id.to_owned(),
        user: get("Name").unwrap_or_else(|| "?".into()),
        kind: get("Type").unwrap_or_else(|| "?".into()),
        class: get("Class").unwrap_or_default(),
        place: get("RemoteHost")
            .or_else(|| get("TTY"))
            .or_else(|| get("Seat"))
            .unwrap_or_else(|| "—".into()),
        remote: get("RemoteHost").is_some(),
        since: get("Timestamp").map(|t| tidy_timestamp(&t)),
        state: get("State").unwrap_or_default(),
    }
}

pub(crate) fn snapshot() -> OsLogins {
    let Some(listing) = run("loginctl", &["list-sessions", "--no-legend"]) else {
        return OsLogins {
            available: false,
            sessions: Vec::new(),
        };
    };
    let sessions = parse_session_ids(&listing)
        .into_iter()
        .take(30)
        .filter_map(|id| {
            let props = run("loginctl", &["show-session", &id]).map(|t| parse_properties(&t))?;
            Some(session_from(&props, &id))
        })
        .collect();
    OsLogins {
        available: true,
        sessions,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn session_ids_are_the_first_column() {
        assert_eq!(
            parse_session_ids(" 3 1000 talha seat0 tty2\n c1 120 gdm seat0\n"),
            ["3", "c1"]
        );
        assert!(parse_session_ids("").is_empty());
    }

    #[test]
    fn a_session_reads_its_user_type_and_place() {
        let props = parse_properties(
            "Name=talha\nType=wayland\nClass=user\nTTY=tty2\nSeat=seat0\nState=active\nTimestamp=Sun 2026-10-04 05:37:12 PKT\nRemoteHost=\n",
        );
        let s = session_from(&props, "3");
        assert_eq!(
            (s.user.as_str(), s.kind.as_str(), s.place.as_str()),
            ("talha", "wayland", "tty2")
        );
        assert!(!s.remote);
        assert_eq!(s.since.as_deref(), Some("2026-10-04 05:37:12 PKT"));
    }

    #[test]
    fn a_remote_session_shows_where_it_comes_from() {
        let props =
            parse_properties("Name=talha\nType=tty\nRemoteHost=192.168.1.50\nState=active\n");
        let s = session_from(&props, "7");
        assert_eq!(s.place, "192.168.1.50");
        assert!(s.remote);
    }

    #[test]
    fn this_machine_lists_its_sessions() {
        let l = snapshot();
        assert!(l.available);
        assert!(!l.sessions.is_empty());
    }
}
