use std::process::Command;

use crate::error::{AppError, Result};

/// The signals the UI may send, by the name it uses.
pub(crate) fn flag(signal: &str) -> Option<&'static str> {
    Some(match signal {
        "term" => "-TERM",
        "kill" => "-KILL",
        "stop" => "-STOP",
        "cont" => "-CONT",
        _ => return None,
    })
}

/// PID 0 and 1 would hit a whole process group or the init system, and the app must not end itself.
pub(crate) fn check_target(pid: u32, own_pid: u32) -> std::result::Result<(), String> {
    if pid <= 1 {
        Err(format!(
            "PID {pid} is the system itself. Signalling it would affect everything."
        ))
    } else if pid == own_pid {
        Err("That is Viewust itself.".to_owned())
    } else {
        Ok(())
    }
}

/// Turns what `kill` printed into something a person can act on.
pub(crate) fn explain(stderr: &str) -> String {
    let text = stderr.to_lowercase();
    if text.contains("not permitted") {
        "Only the owner of a program (or an administrator) can stop it.".to_owned()
    } else if text.contains("no such process") {
        "That program has already ended.".to_owned()
    } else {
        crate::common::elevate::first_line(stderr)
    }
}

pub(crate) fn send(pid: u32, signal: &str) -> Result<()> {
    let flag = flag(signal).ok_or_else(|| AppError::Other(format!("Unknown signal “{signal}”")))?;
    check_target(pid, std::process::id()).map_err(AppError::Other)?;
    let out = Command::new("kill")
        .args([flag, "--", &pid.to_string()])
        .output()
        .map_err(|_| AppError::MissingTool("kill"))?;
    if out.status.success() {
        Ok(())
    } else {
        Err(AppError::Other(explain(&String::from_utf8_lossy(
            &out.stderr,
        ))))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_known_signals_are_accepted() {
        assert_eq!(flag("term"), Some("-TERM"));
        assert_eq!(flag("kill"), Some("-KILL"));
        assert_eq!(flag("stop"), Some("-STOP"));
        assert_eq!(flag("cont"), Some("-CONT"));
        assert_eq!(flag("9; rm -rf /"), None);
        assert_eq!(flag(""), None);
    }

    #[test]
    fn the_system_and_the_app_itself_are_protected() {
        assert!(check_target(0, 500).is_err());
        assert!(check_target(1, 500).is_err());
        assert!(check_target(500, 500).is_err());
        assert!(check_target(501, 500).is_ok());
    }

    #[test]
    fn kill_errors_become_plain_sentences() {
        assert!(explain("kill: (1): Operation not permitted").contains("owner"));
        assert!(explain("kill: (999): No such process").contains("already ended"));
        assert_eq!(explain("kill: odd"), "kill: odd");
    }

    #[test]
    fn an_unknown_signal_is_refused_before_anything_is_sent() {
        assert!(send(4_000_000, "bogus")
            .unwrap_err()
            .to_string()
            .contains("Unknown"));
    }

    #[test]
    fn a_real_program_can_be_terminated() {
        use std::os::unix::process::ExitStatusExt;
        let mut child = Command::new("sleep").arg("30").spawn().unwrap();
        send(child.id(), "term").unwrap();
        assert_eq!(child.wait().unwrap().signal(), Some(15));
    }

    #[test]
    fn signalling_a_program_that_ended_explains_itself() {
        let mut child = Command::new("true").spawn().unwrap();
        let pid = child.id();
        child.wait().unwrap();
        // The PID is gone (reaped), so `kill` has nothing to hit.
        let err = send(pid, "term").unwrap_err().to_string();
        assert!(err.contains("already ended"), "{err}");
    }
}
