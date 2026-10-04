use super::model::OsLogs;
use crate::common::cmd::{run, run_any};

/// "Archived and active journals take up 3.9G in the file system." → "3.9G".
pub(crate) fn parse_disk_usage(text: &str) -> Option<String> {
    text.split_whitespace()
        .skip_while(|w| *w != "up")
        .nth(1)
        .map(str::to_owned)
}

const ERROR_CAP: usize = 200;

pub(crate) fn snapshot() -> OsLogs {
    let Some(usage) = run_any("journalctl", &["--disk-usage"]) else {
        return OsLogs {
            available: false,
            note: Some("journalctl isn't installed, so there is no journal to read.".into()),
            size: None,
            boots: None,
            errors: Vec::new(),
            truncated: false,
        };
    };
    let size = parse_disk_usage(&usage);
    let boots = run("journalctl", &["--list-boots", "-q", "--no-pager"])
        .map(|t| t.lines().filter(|l| !l.trim().is_empty()).count());
    let cap_arg = (ERROR_CAP + 1).to_string();
    let errors: Vec<String> = run(
        "journalctl",
        &[
            "-b",
            "-p",
            "3",
            "-q",
            "--no-pager",
            "-n",
            &cap_arg,
            "-o",
            "short",
        ],
    )
    .map(|t| t.lines().map(str::to_owned).collect())
    .unwrap_or_default();
    let truncated = errors.len() > ERROR_CAP;
    let note = if size.is_none() && boots.is_none() {
        Some(
            "The journal couldn't be read. Members of the adm group can read all of it.".to_owned(),
        )
    } else {
        None
    };
    OsLogs {
        available: true,
        size,
        boots,
        errors: errors.into_iter().take(ERROR_CAP).collect(),
        truncated,
        note,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_journal_size_is_the_word_after_up() {
        assert_eq!(
            parse_disk_usage("Archived and active journals take up 3.9G in the file system.\n")
                .as_deref(),
            Some("3.9G")
        );
        assert_eq!(parse_disk_usage("nothing here"), None);
    }

    #[test]
    fn this_machine_reports_its_journal() {
        let l = snapshot();
        assert!(l.available);
    }
}
