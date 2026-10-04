use super::devices::{can_mount, lsblk};
use crate::error::{AppError, Result};

/// `udisksctl` answers "Mounted /dev/sda2 at /media/me/New Volume."; this picks out the place.
pub(crate) fn parse_mounted_at(stdout: &str) -> Option<String> {
    let (_, rest) = stdout.trim().split_once(" at ")?;
    let path = rest.trim().trim_end_matches('.');
    path.starts_with('/').then(|| path.to_owned())
}

/// Mounts a filesystem the way a desktop does, so it can be browsed. Returns where it landed.
pub(crate) fn mount(device: &str) -> Result<String> {
    can_mount(&lsblk(), device).map_err(AppError::Other)?;
    let out = std::process::Command::new("udisksctl")
        .args(["mount", "-b", device])
        .output()
        .map_err(|_| AppError::MissingTool("udisksctl"))?;
    if !out.status.success() {
        let stderr = String::from_utf8_lossy(&out.stderr);
        return Err(AppError::Other(crate::common::elevate::first_line(&stderr)));
    }
    parse_mounted_at(&String::from_utf8_lossy(&out.stdout))
        .ok_or_else(|| AppError::Other("It mounted, but I couldn't tell where".into()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_mount_point_is_read_from_the_answer() {
        assert_eq!(
            parse_mounted_at("Mounted /dev/sda2 at /media/me/New Volume.\n").as_deref(),
            Some("/media/me/New Volume")
        );
        assert_eq!(
            parse_mounted_at("Mounted /dev/sdb1 at /run/media/me/DATA").as_deref(),
            Some("/run/media/me/DATA")
        );
        assert_eq!(parse_mounted_at("Error mounting"), None);
    }

    #[test]
    fn a_path_that_is_not_a_device_is_refused_before_anything_runs() {
        let err = mount("/etc/passwd").unwrap_err().to_string();
        assert!(err.contains("isn't a storage device"), "{err}");
    }
}
