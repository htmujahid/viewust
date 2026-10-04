use std::path::Path;

use super::model::{CgroupRow, Cgroups};

pub(crate) fn parse_controllers(text: &str) -> Vec<String> {
    text.split_whitespace().map(str::to_owned).collect()
}

/// How many control groups exist under `root`. Capped: the exact figure matters less than
/// its size, and a busy container host can hold thousands.
pub(crate) fn count_groups(root: &Path, cap: usize) -> usize {
    let mut count = 0;
    let mut stack = vec![root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        let Ok(entries) = std::fs::read_dir(&dir) else {
            continue;
        };
        for entry in entries.flatten() {
            if entry.file_type().is_ok_and(|t| t.is_dir()) {
                count += 1;
                if count >= cap {
                    return cap;
                }
                stack.push(entry.path());
            }
        }
    }
    count
}

pub(crate) const CAP: usize = 5000;

/// One `key value` line out of files like `cgroup.stat`.
pub(crate) fn stat_value(text: &str, key: &str) -> Option<u64> {
    text.lines()
        .find_map(|l| l.strip_prefix(key))
        .and_then(|rest| rest.trim().parse().ok())
}

fn number(path: std::path::PathBuf) -> Option<u64> {
    std::fs::read_to_string(path).ok()?.trim().parse().ok()
}

/// The groups sitting directly under the root: the big slices everything lives in.
pub(crate) fn top_level(root: &Path) -> Vec<CgroupRow> {
    let mut rows: Vec<CgroupRow> = std::fs::read_dir(root)
        .into_iter()
        .flatten()
        .flatten()
        .filter(|e| e.file_type().is_ok_and(|t| t.is_dir()))
        .map(|e| {
            let dir = e.path();
            CgroupRow {
                name: e.file_name().to_string_lossy().into_owned(),
                pids: number(dir.join("pids.current")),
                memory: number(dir.join("memory.current")),
                groups: std::fs::read_to_string(dir.join("cgroup.stat"))
                    .ok()
                    .and_then(|t| stat_value(&t, "nr_descendants")),
            }
        })
        .collect();
    rows.sort_by(|a, b| b.memory.cmp(&a.memory).then_with(|| a.name.cmp(&b.name)));
    rows
}

pub(crate) fn snapshot() -> Cgroups {
    let root = Path::new("/sys/fs/cgroup");
    let controllers = std::fs::read_to_string(root.join("cgroup.controllers"))
        .map(|t| parse_controllers(&t))
        .ok();
    let version = match &controllers {
        Some(_) => "v2 (unified)",
        None if root.exists() => "v1 (legacy)",
        None => "none",
    };
    Cgroups {
        version,
        controllers: controllers.unwrap_or_default(),
        groups: count_groups(root, CAP),
        capped: false,
        top: top_level(root),
    }
    .cap()
}

impl Cgroups {
    fn cap(mut self) -> Self {
        self.capped = self.groups >= CAP;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn controllers_are_split_on_whitespace() {
        assert_eq!(
            parse_controllers("cpuset cpu io memory pids\n"),
            ["cpuset", "cpu", "io", "memory", "pids"]
        );
        assert!(parse_controllers("").is_empty());
    }

    #[test]
    fn groups_are_counted_through_the_tree_up_to_the_cap() {
        let root = std::env::temp_dir().join(format!("viewust-cg-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        for p in ["a/a1", "a/a2", "b"] {
            std::fs::create_dir_all(root.join(p)).unwrap();
        }
        std::fs::write(root.join("a/file"), "x").unwrap();
        assert_eq!(count_groups(&root, 100), 4);
        assert_eq!(count_groups(&root, 2), 2);
        std::fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn this_machine_reports_its_cgroups() {
        let c = snapshot();
        assert!(c.version.starts_with('v'));
        assert!(c.groups > 0);
    }

    #[test]
    fn stat_lines_give_their_number() {
        let t = "nr_descendants 143\nnr_dying_descendants 2\n";
        assert_eq!(stat_value(t, "nr_descendants"), Some(143));
        assert_eq!(stat_value(t, "nr_dying_descendants"), Some(2));
        assert_eq!(stat_value(t, "missing"), None);
    }

    #[test]
    fn top_level_groups_are_read_and_sorted_by_memory() {
        let root = std::env::temp_dir().join(format!("viewust-cgtop-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        for (name, mem, pids) in [("user.slice", "900", "12"), ("system.slice", "4000", "40")] {
            let d = root.join(name);
            std::fs::create_dir_all(&d).unwrap();
            std::fs::write(d.join("memory.current"), mem).unwrap();
            std::fs::write(d.join("pids.current"), pids).unwrap();
            std::fs::write(d.join("cgroup.stat"), "nr_descendants 3\n").unwrap();
        }
        std::fs::create_dir_all(root.join("empty.scope")).unwrap();
        let rows = top_level(&root);
        assert_eq!(rows.len(), 3);
        assert_eq!(rows[0].name, "system.slice");
        assert_eq!(rows[0].memory, Some(4000));
        assert_eq!(rows[1].pids, Some(12));
        assert_eq!(
            rows[2],
            CgroupRow {
                name: "empty.scope".into(),
                pids: None,
                memory: None,
                groups: None
            }
        );
        std::fs::remove_dir_all(&root).unwrap();
    }
}
