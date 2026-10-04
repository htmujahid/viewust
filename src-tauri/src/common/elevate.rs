use std::path::Path;
use std::process::Command;

use crate::error::{AppError, Result};

/// Runs a program as administrator through `pkexec`, which asks the desktop for permission.
/// The program is looked up in `candidates` because `pkexec` wants an absolute path.
pub(crate) fn run(candidates: &[&str], tool: &'static str, args: &[&str]) -> Result<String> {
    let program = candidates
        .iter()
        .find(|p| Path::new(p).exists())
        .ok_or(AppError::MissingTool(tool))?;
    let out = Command::new("pkexec")
        .arg(program)
        .args(args)
        .output()
        .map_err(|e| AppError::Other(format!("could not ask for permission: {e}")))?;
    if out.status.success() {
        return Ok(String::from_utf8_lossy(&out.stdout).into_owned());
    }
    Err(match out.status.code() {
        Some(126) | Some(127) => AppError::PermissionDenied,
        _ => AppError::Other(first_line(&String::from_utf8_lossy(&out.stderr))),
    })
}

/// The part of a failed command's message worth showing: its first non-empty line.
pub(crate) fn first_line(text: &str) -> String {
    text.lines()
        .map(str::trim)
        .find(|l| !l.is_empty())
        .unwrap_or("Permission was declined or the command failed")
        .to_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_first_meaningful_line_of_stderr_is_kept() {
        assert_eq!(
            first_line("\n  Failed to stop x: busy\nmore"),
            "Failed to stop x: busy"
        );
        assert_eq!(
            first_line(""),
            "Permission was declined or the command failed"
        );
    }

    #[test]
    fn a_missing_program_is_reported_by_name() {
        let err = run(&["/nonexistent/tool"], "tool", &[]).unwrap_err();
        assert_eq!(err.to_string(), "tool is not installed");
    }
}
