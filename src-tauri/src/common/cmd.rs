use std::process::Command;

pub(crate) fn run(program: &str, args: &[&str]) -> Option<String> {
    let out = Command::new(program).args(args).output().ok()?;
    out.status
        .success()
        .then(|| String::from_utf8_lossy(&out.stdout).into_owned())
}

/// Like `run`, but keeps the output of a command that exits non-zero on purpose
/// (`systemd-detect-virt` prints "none" and fails on a physical machine).
pub(crate) fn run_any(program: &str, args: &[&str]) -> Option<String> {
    let out = Command::new(program).args(args).output().ok()?;
    Some(String::from_utf8_lossy(&out.stdout).into_owned())
}
