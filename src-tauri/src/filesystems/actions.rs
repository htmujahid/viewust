use std::process::Command;

use super::mounts::{parse_mountinfo, Mount};
use crate::common::cmd::run;
use crate::common::elevate;
use crate::error::{AppError, Result};

/// Folders the system needs to keep running; unmounting one is never what anyone wants.
const KEEP: &[&str] = &[
    "/",
    "/boot",
    "/boot/efi",
    "/usr",
    "/var",
    "/etc",
    "/proc",
    "/sys",
    "/dev",
    "/run",
];

pub(crate) fn protected(mount: &str) -> bool {
    KEEP.contains(&mount)
}

fn find(mount: &str) -> Result<Mount> {
    let text = std::fs::read_to_string("/proc/self/mountinfo")
        .map_err(|e| AppError::Other(format!("Couldn't read the mount table: {e}")))?;
    parse_mountinfo(&text)
        .into_iter()
        .rev()
        .find(|m| m.target == mount)
        .ok_or_else(|| AppError::Other(format!("{mount} is no longer mounted")))
}

pub(crate) fn open(mount: &str) -> Result<()> {
    find(mount)?;
    let mut child = Command::new("xdg-open")
        .arg(mount)
        .spawn()
        .map_err(|_| AppError::MissingTool("xdg-open"))?;
    // Reap it in the background so the file manager doesn't leave a zombie behind.
    std::thread::spawn(move || {
        let _ = child.wait();
    });
    Ok(())
}

pub(crate) fn unmount(mount: &str) -> Result<()> {
    if protected(mount) {
        return Err(AppError::Other(format!(
            "{mount} is needed by the system and can't be unmounted here."
        )));
    }
    let m = find(mount)?;
    // `udisksctl` is how desktops unmount drives without a password; it only knows block devices.
    if m.source.starts_with("/dev/") {
        if let Some(out) = run("udisksctl", &["unmount", "-b", &m.source]) {
            let _ = out;
            return Ok(());
        }
    }
    elevate::run(
        &["/usr/bin/umount", "/bin/umount"],
        "umount",
        &["--", mount],
    )
    .map(drop)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn system_folders_are_protected_and_user_drives_are_not() {
        assert!(protected("/"));
        assert!(protected("/boot/efi"));
        assert!(!protected("/media/me/STICK"));
        assert!(!protected("/mnt/nas"));
        assert!(!protected("/home"));
    }

    #[test]
    fn something_that_is_not_mounted_is_refused() {
        let err = unmount("/definitely/not/a/mount").unwrap_err();
        assert!(err.to_string().contains("no longer mounted"));
        assert!(open("/definitely/not/a/mount").is_err());
    }
}
