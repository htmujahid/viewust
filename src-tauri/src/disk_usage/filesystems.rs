use super::mounts::{parse_mountinfo, visible};
use super::usage::{parse_df, Usage, DF_COLUMNS};
use crate::common::cmd::run;
use crate::error::{AppError, Result};
use serde::Serialize;
use std::collections::HashMap;

#[derive(Serialize, Debug)]
pub struct FsRow {
    pub(crate) mount: String,
    pub(crate) source: String,
    pub(crate) fstype: String,
    pub(crate) size: Option<u64>,
    pub(crate) used: Option<u64>,
    pub(crate) available: Option<u64>,
    /// No space of its own: a kernel view or memory-backed mount
    pub(crate) pseudo: bool,
}

#[derive(Serialize)]
pub struct Filesystems {
    pub(crate) mounted: usize,
    pub(crate) real: usize,
    pub(crate) rows: Vec<FsRow>,
}

pub(crate) fn build(
    mounts: Vec<super::mounts::Mount>,
    usage: &HashMap<String, Usage>,
) -> Filesystems {
    let mut rows: Vec<FsRow> = mounts
        .into_iter()
        .map(|m| {
            let space = usage.get(&m.target).copied().unwrap_or_default();
            FsRow {
                pseudo: space.size.is_none(),
                mount: m.target,
                source: m.source,
                fstype: m.fstype,
                size: space.size,
                used: space.used,
                available: space.available,
            }
        })
        .collect();
    // Real filesystems first, biggest first; the kernel's own mounts at the bottom.
    rows.sort_by(|a, b| {
        a.pseudo
            .cmp(&b.pseudo)
            .then(b.size.cmp(&a.size))
            .then_with(|| a.mount.cmp(&b.mount))
    });
    Filesystems {
        mounted: rows.len(),
        real: rows.iter().filter(|r| !r.pseudo).count(),
        rows,
    }
}

pub(crate) fn snapshot() -> Result<Filesystems> {
    let text = std::fs::read_to_string("/proc/self/mountinfo")
        .map_err(|e| AppError::Other(format!("Couldn't read the mount table: {e}")))?;
    let usage: HashMap<String, Usage> = run("df", &["-B1", "-a", "-l", DF_COLUMNS])
        .map(|t| parse_df(&t))
        .unwrap_or_default();
    Ok(build(visible(parse_mountinfo(&text)), &usage))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn real_filesystems_come_first_sorted_by_size() {
        let mounts = parse_mountinfo(
            "1 0 0:1 / /proc rw - proc proc rw\n\
             2 0 8:2 / / rw - ext4 /dev/sda2 rw\n\
             3 0 8:3 / /home rw - ext4 /dev/sda3 rw\n",
        );
        let usage = HashMap::from([
            (
                "/".to_owned(),
                Usage {
                    size: Some(100),
                    used: Some(40),
                    available: Some(55),
                },
            ),
            (
                "/home".to_owned(),
                Usage {
                    size: Some(500),
                    used: Some(10),
                    available: Some(490),
                },
            ),
            ("/proc".to_owned(), Usage::default()),
        ]);
        let fs = build(mounts, &usage);
        let order: Vec<&str> = fs.rows.iter().map(|r| r.mount.as_str()).collect();
        assert_eq!(order, ["/home", "/", "/proc"]);
        assert_eq!((fs.mounted, fs.real), (3, 2));
        assert!(fs.rows[2].pseudo);
        assert_eq!(fs.rows[1].used, Some(40));
    }

    #[test]
    fn this_machine_lists_its_mounts_once_each() {
        let fs = snapshot().unwrap();
        assert!(fs.real >= 1);
        let mut seen = std::collections::HashSet::new();
        for r in &fs.rows {
            assert!(seen.insert(&r.mount), "{} twice", r.mount);
        }
    }
}
