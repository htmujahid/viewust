//! One process in depth, read from `/proc/<pid>`.

use super::model::*;
use super::service::ProcessService;
use crate::common::format::*;
use crate::common::sysfs::*;
use crate::common::Details;
use std::collections::HashMap;
use sysinfo::Pid;

fn kb_map(text: &str) -> HashMap<String, u64> {
    text.lines()
        .filter_map(|l| {
            let (k, v) = l.split_once(':')?;
            let n = v.split_whitespace().next()?.parse::<u64>().ok()?;
            let bytes = if v.trim().ends_with("kB") {
                n * 1024
            } else {
                n
            };
            Some((k.to_owned(), bytes))
        })
        .collect()
}

/// Fields of /proc/<pid>/stat after the command name (which may hold spaces).
fn stat_fields(pid: u32) -> Option<Vec<String>> {
    let text = std::fs::read_to_string(format!("/proc/{pid}/stat")).ok()?;
    let rest = &text[text.rfind(')')? + 2..];
    Some(rest.split_whitespace().map(str::to_owned).collect())
}

impl ProcessService {
    /// One process in depth, read from `/proc/<pid>`.
    pub fn detail(&self, pid: u32) -> ProcessDetail {
        let dir = format!("/proc/{pid}");
        let Ok(status_text) = std::fs::read_to_string(format!("{dir}/status")) else {
            return ProcessDetail {
                pid,
                running: false,
                restricted: false,
                memory: None,
                regions: vec![],
                children: vec![],
                details: vec![],
            };
        };
        let status = kb_map(&status_text);
        let field = |k: &str| {
            status_text
                .lines()
                .find_map(|l| l.strip_prefix(&format!("{k}:")))
                .map(|v| v.trim().to_owned())
        };
        let bytes = |k: &str| status.get(k).copied().unwrap_or(0);

        let mut memory = MemoryBreakdown {
            requested: bytes("VmSize"),
            peak_requested: bytes("VmPeak"),
            resident: bytes("VmRSS"),
            peak_resident: bytes("VmHWM"),
            anonymous: bytes("RssAnon"),
            file_backed: bytes("RssFile"),
            shared: bytes("RssShmem"),
            swapped: bytes("VmSwap"),
            data: bytes("VmData"),
            stack: bytes("VmStk"),
            code: bytes("VmExe"),
            libraries: bytes("VmLib"),
            page_tables: bytes("VmPTE"),
            locked: bytes("VmLck"),
            ..Default::default()
        };

        let mut restricted = false;
        if let Some(r) = read(format!("{dir}/smaps_rollup")) {
            let m = kb_map(&r);
            memory.proportional = m.get("Pss").copied();
            memory.private = Some(
                m.get("Private_Clean").copied().unwrap_or(0)
                    + m.get("Private_Dirty").copied().unwrap_or(0),
            );
            memory.shared_pages = Some(
                m.get("Shared_Clean").copied().unwrap_or(0)
                    + m.get("Shared_Dirty").copied().unwrap_or(0),
            );
        } else {
            restricted = true;
        }

        // Largest memory regions, merged by what they map.
        let mut regions: Vec<Region> = Vec::new();
        if let Ok(smaps) = std::fs::read_to_string(format!("{dir}/smaps")) {
            let mut current: Option<String> = None;
            let mut map: HashMap<String, (u64, u64)> = HashMap::new();
            for line in smaps.lines() {
                let first = line.split_whitespace().next().unwrap_or("");
                if first.contains('-') && !first.ends_with(':') {
                    let name = line
                        .split_whitespace()
                        .nth(5)
                        .map(str::to_owned)
                        .unwrap_or_else(|| "(anonymous)".into());
                    current = Some(name);
                } else if let Some(name) = &current {
                    let entry = map.entry(name.clone()).or_default();
                    if let Some(v) = line.strip_prefix("Size:") {
                        entry.0 +=
                            v.trim().trim_end_matches(" kB").parse::<u64>().unwrap_or(0) * 1024;
                    } else if let Some(v) = line.strip_prefix("Rss:") {
                        entry.1 +=
                            v.trim().trim_end_matches(" kB").parse::<u64>().unwrap_or(0) * 1024;
                    }
                }
            }
            regions = map
                .into_iter()
                .map(|(name, (size, resident))| Region {
                    name,
                    size,
                    resident,
                })
                .collect();
            regions.sort_by(|a, b| b.resident.cmp(&a.resident).then(b.size.cmp(&a.size)));
            regions.truncate(10);
        }

        let stat = stat_fields(pid);
        let sf = |i: usize| {
            stat.as_ref()
                .and_then(|f| f.get(i))
                .and_then(|v| v.parse::<u64>().ok())
        };
        // fields counted from 3 (state); index = field number - 3
        let (minflt, majflt, utime, stime) = (sf(7), sf(9), sf(11), sf(12));

        let mut d = Details::new();
        let name = field("Name").unwrap_or_default();
        d.add("Process", "Name", &name);
        d.add("Process", "ID (PID)", pid.to_string());
        d.add_opt("Process", "State", field("State"));
        d.add_opt("Process", "Threads", field("Threads"));

        let ppid: Option<u32> = field("PPid").and_then(|p| p.parse().ok());
        let (children, started, user, cmd_name_of_parent) = self.with_fresh(|st| {
            let mut children: Vec<Child> = st
                .sys
                .processes()
                .values()
                .filter(|p| {
                    p.parent().map(|x| x.as_u32()) == Some(pid) && p.thread_kind().is_none()
                })
                .map(|p| Child {
                    pid: p.pid().as_u32(),
                    name: p.name().to_string_lossy().into_owned(),
                    memory: p.memory(),
                })
                .collect();
            // hash-map order is arbitrary; show the biggest first, always in the same order
            children.sort_by(|a, b| b.memory.cmp(&a.memory).then(a.pid.cmp(&b.pid)));
            let me = st.sys.process(Pid::from_u32(pid));
            (
                children,
                me.map(|p| (p.start_time(), p.run_time())),
                me.and_then(|p| p.user_id())
                    .and_then(|u| st.users.get_user_by_id(u))
                    .map(|u| u.name().to_owned()),
                ppid.and_then(|pp| st.sys.process(Pid::from_u32(pp)))
                    .map(|p| p.name().to_string_lossy().into_owned()),
            )
        });
        d.add_opt("Process", "Owner", user);
        if let Some(pp) = ppid {
            d.add(
                "Process",
                "Started by",
                format!(
                    "{} ({pp})",
                    cmd_name_of_parent.unwrap_or_else(|| "?".into())
                ),
            );
        }
        if let Some((_, run)) = started {
            d.add("Process", "Running for", duration(run));
        }

        let cmdline = std::fs::read(format!("{dir}/cmdline"))
            .map(|b| {
                String::from_utf8_lossy(&b)
                    .replace('\0', " ")
                    .trim()
                    .to_owned()
            })
            .unwrap_or_default();
        d.add(
            "Command",
            "Command line",
            if cmdline.chars().count() > 700 {
                format!("{}…", cmdline.chars().take(700).collect::<String>())
            } else {
                cmdline
            },
        );
        d.add_opt(
            "Command",
            "Program",
            std::fs::read_link(format!("{dir}/exe"))
                .ok()
                .map(|p| p.to_string_lossy().into_owned()),
        );
        d.add_opt(
            "Command",
            "Working folder",
            std::fs::read_link(format!("{dir}/cwd"))
                .ok()
                .map(|p| p.to_string_lossy().into_owned()),
        );
        d.add_opt(
            "Command",
            "Control group",
            read(format!("{dir}/cgroup")).and_then(|c| {
                let path = c.lines().next_back()?.split(':').next_back()?.to_owned();
                let parts: Vec<&str> = path.split('/').filter(|p| !p.is_empty()).collect();
                Some(parts[parts.len().saturating_sub(2)..].join(" / ")).filter(|s| !s.is_empty())
            }),
        );

        d.add_opt(
            "CPU",
            "CPU time used",
            match (utime, stime) {
                (Some(u), Some(s)) => Some(format!(
                    "{} (user {} · system {})",
                    secs(u + s),
                    secs(u),
                    secs(s)
                )),
                _ => None,
            },
        );
        d.add_opt(
            "CPU",
            "Priority / nice",
            match (sf(15), sf(16)) {
                (Some(p), Some(n)) => Some(format!("{p} / {}", n as i64 as i32)),
                _ => None,
            },
        );
        d.add_opt("CPU", "Last ran on core", sf(36).map(|c| c.to_string()));
        d.add_opt(
            "CPU",
            "Context switches",
            match (
                field("voluntary_ctxt_switches"),
                field("nonvoluntary_ctxt_switches"),
            ) {
                (Some(v), Some(n)) => Some(format!("{v} voluntary · {n} forced")),
                _ => None,
            },
        );

        d.add(
            "Memory requests",
            "Asked for (virtual)",
            format_bytes(memory.requested),
        );
        d.add(
            "Memory requests",
            "Peak asked for",
            format_bytes(memory.peak_requested),
        );
        d.add(
            "Memory requests",
            "In RAM now (resident)",
            format_bytes(memory.resident),
        );
        d.add(
            "Memory requests",
            "Peak in RAM",
            format_bytes(memory.peak_resident),
        );
        if memory.requested > 0 {
            d.add(
                "Memory requests",
                "Actually used",
                format!(
                    "{:.1}% of what it asked for",
                    memory.resident as f64 / memory.requested as f64 * 100.0
                ),
            );
        }
        d.add(
            "Memory requests",
            "Moved to swap",
            format_bytes(memory.swapped),
        );
        d.add(
            "Memory requests",
            "Locked in RAM",
            format_bytes(memory.locked),
        );
        d.add_opt(
            "Page faults",
            "Minor (satisfied from memory)",
            minflt.map(|n| n.to_string()),
        );
        d.add_opt(
            "Page faults",
            "Major (had to read from disk)",
            majflt.map(|n| n.to_string()),
        );
        d.add("Segments", "Heap and data", format_bytes(memory.data));
        d.add("Segments", "Stack", format_bytes(memory.stack));
        d.add("Segments", "Program code", format_bytes(memory.code));
        d.add(
            "Segments",
            "Shared libraries",
            format_bytes(memory.libraries),
        );
        d.add("Segments", "Page tables", format_bytes(memory.page_tables));

        if let Some(io) = read(format!("{dir}/io")) {
            let m = kb_map(&io); // values here are plain byte counts
            let g = |k: &str| m.get(k).copied().map(format_bytes);
            d.add_opt("Disk and I/O", "Read from disk", g("read_bytes"));
            d.add_opt("Disk and I/O", "Written to disk", g("write_bytes"));
            d.add_opt("Disk and I/O", "Read by calls (incl. cache)", g("rchar"));
            d.add_opt("Disk and I/O", "Written by calls (incl. cache)", g("wchar"));
        } else {
            restricted = true;
        }
        if let Ok(fds) = std::fs::read_dir(format!("{dir}/fd")) {
            d.add("Files", "Open files and sockets", fds.count().to_string());
        }
        d.add_opt(
            "Files",
            "Open file limit",
            read(format!("{dir}/limits")).and_then(|l| {
                l.lines()
                    .find(|x| x.starts_with("Max open files"))
                    .map(|x| x.split_whitespace().rev().nth(1).unwrap_or("?").to_owned())
            }),
        );
        if restricted {
            d.add(
            "Access",
            "Restricted",
            "This process belongs to another user, so the system only shares its basic status. \
             The memory map and disk I/O are hidden.",
        );
        }

        ProcessDetail {
            pid,
            running: true,
            restricted,
            memory: Some(memory),
            regions,
            children,
            details: d.finish(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn status_values_are_read_in_bytes() {
        let m = kb_map("Name:\tfirefox\nVmRSS:\t  1024 kB\nThreads:\t5\n");
        assert_eq!(m["VmRSS"], 1024 * 1024);
        assert_eq!(m["Threads"], 5);
        assert!(!m.contains_key("Name")); // not a number
    }
}
