use super::model::Namespaces;

pub(crate) const KINDS: &[&str] = &["pid", "net", "mnt", "user", "uts", "ipc", "cgroup", "time"];

pub(crate) fn parse_link(target: &str) -> Option<(String, u64)> {
    let (kind, rest) = target.split_once(":[")?;
    let id = rest.strip_suffix(']')?.parse().ok()?;
    Some((kind.to_owned(), id))
}

#[cfg(target_os = "linux")]
pub(crate) fn snapshot() -> Namespaces {
    linux::snapshot()
}

#[cfg(not(target_os = "linux"))]
pub(crate) fn snapshot() -> Namespaces {
    Namespaces {
        supported: false,
        note: Some(
            "Namespaces are a Linux feature. This system isolates programs in other ways, \
             so there is nothing to list here."
                .into(),
        ),
        inspected: 0,
        total: 0,
        namespaces: Vec::new(),
    }
}

#[cfg(target_os = "linux")]
mod linux {
    use std::collections::HashMap;
    use std::os::unix::fs::MetadataExt;

    use sysinfo::Users;

    use super::{parse_link, KINDS};
    use crate::processes::model::{NamespaceRow, Namespaces, NsProcess};

    const SAMPLE: usize = 8;

    fn read_ns(pid: &str, kind: &str) -> Option<u64> {
        let link = std::fs::read_link(format!("/proc/{pid}/ns/{kind}")).ok()?;
        parse_link(&link.to_string_lossy()).map(|(_, id)| id)
    }

    pub(super) fn snapshot() -> Namespaces {
        let users = Users::new_with_refreshed_list();
        let mine: HashMap<&str, u64> = KINDS
            .iter()
            .filter_map(|k| Some((*k, read_ns("self", k)?)))
            .collect();

        let mut pids: Vec<u32> = std::fs::read_dir("/proc")
            .into_iter()
            .flatten()
            .flatten()
            .filter_map(|e| e.file_name().to_string_lossy().parse().ok())
            .collect();
        pids.sort_unstable();

        let mut table: HashMap<(String, u64), NamespaceRow> = HashMap::new();
        let mut inspected = 0;
        for pid in &pids {
            let id = pid.to_string();
            if read_ns(&id, "pid").is_none() {
                continue;
            }
            inspected += 1;
            let name = std::fs::read_to_string(format!("/proc/{id}/comm"))
                .map(|s| s.trim().to_owned())
                .unwrap_or_default();
            let user = std::fs::metadata(format!("/proc/{id}"))
                .ok()
                .and_then(|m| {
                    users
                        .list()
                        .iter()
                        .find(|u| **u.id() == m.uid())
                        .map(|u| u.name().to_owned())
                })
                .unwrap_or_default();
            for kind in KINDS {
                let Some(ns) = read_ns(&id, kind) else {
                    continue;
                };
                let row = table
                    .entry((kind.to_string(), ns))
                    .or_insert_with(|| NamespaceRow {
                        kind: kind.to_string(),
                        id: ns,
                        processes: 0,
                        sample: Vec::new(),
                        current: mine.get(kind) == Some(&ns),
                    });
                row.processes += 1;
                if row.sample.len() < SAMPLE {
                    row.sample.push(NsProcess {
                        pid: *pid,
                        name: name.clone(),
                        user: user.clone(),
                    });
                }
            }
        }

        let order = |k: &str| KINDS.iter().position(|x| *x == k).unwrap_or(KINDS.len());
        let mut namespaces: Vec<NamespaceRow> = table.into_values().collect();
        namespaces.sort_by(|a, b| {
            order(&a.kind)
                .cmp(&order(&b.kind))
                .then(b.processes.cmp(&a.processes))
                .then(a.id.cmp(&b.id))
        });

        let total = pids.len();
        Namespaces {
            supported: true,
            note: (inspected < total).then(|| {
                format!(
                    "Read {inspected} of {total} processes. The rest belong to other users, and \
                     only an administrator can look inside them."
                )
            }),
            inspected,
            total,
            namespaces,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn links_give_the_kind_and_inode() {
        assert_eq!(
            parse_link("net:[4026531833]"),
            Some(("net".into(), 4026531833))
        );
        assert_eq!(parse_link("pid:[1]"), Some(("pid".into(), 1)));
        assert_eq!(parse_link("garbage"), None);
        assert_eq!(parse_link("net:[x]"), None);
    }

    #[test]
    fn every_kind_has_a_slot_in_the_display_order() {
        assert_eq!(KINDS.len(), 8);
        assert!(KINDS.contains(&"mnt") && KINDS.contains(&"user"));
    }
}
