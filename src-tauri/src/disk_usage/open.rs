use std::process::Command;

use crate::error::{AppError, Result};

/// Hands a folder or file to the desktop's default program for it.
pub(crate) fn open(path: &str) -> Result<()> {
    let target = std::path::Path::new(path);
    if !target.is_absolute() || !target.exists() {
        return Err(AppError::Other(format!("{path} no longer exists")));
    }
    let mut child = Command::new("xdg-open")
        .arg(path)
        .spawn()
        .map_err(|_| AppError::MissingTool("xdg-open"))?;
    // Reap it in the background so the file manager doesn't leave a zombie behind.
    std::thread::spawn(move || {
        let _ = child.wait();
    });
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn something_that_is_gone_or_not_a_full_path_is_refused() {
        assert!(open("/definitely/not/here").is_err());
        assert!(open("relative/path").is_err());
    }
}
