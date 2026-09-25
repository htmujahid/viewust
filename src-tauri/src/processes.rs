//! Running processes and how each one uses memory.
//!
//! The list comes from `sysinfo` (kept alive between calls so CPU usage can be
//! measured over time). The detail view reads `/proc/<pid>` directly. A process
//! owned by another user exposes only basic facts, so everything beyond that is
//! optional and the result says when it was restricted.

use crate::detail::{format_bytes, read, Detail, Details};
use serde::Serialize;
use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};
use std::time::Duration;
use sysinfo::{Pid, ProcessRefreshKind, ProcessesToUpdate, System, UpdateKind, Users};

struct State {
    sys: System,
    users: Users,
}

static STATE: OnceLock<Mutex<State>> = OnceLock::new();

fn refresh_kind() -> ProcessRefreshKind {
    ProcessRefreshKind::nothing()
        .with_cpu()
        .with_memory()
        .with_user(UpdateKind::OnlyIfNotSet)
        .with_tasks()
}

fn refreshed<R>(f: impl FnOnce(&State) -> R) -> R {
    let state = STATE.get_or_init(|| {
        let mut sys = System::new();
        sys.refresh_cpu_all();
        sys.refresh_processes_specifics(ProcessesToUpdate::All, true, refresh_kind());
        // CPU usage is a difference between two samples; take a first one now
        // so the very first screen already has numbers.
        std::thread::sleep(Duration::from_millis(300));
        Mutex::new(State { sys, users: Users::new_with_refreshed_list() })
    });
    let mut guard = state.lock().unwrap_or_else(|e| e.into_inner());
    guard.sys.refresh_cpu_all();
    guard.sys.refresh_memory();
    guard.sys.refresh_processes_specifics(ProcessesToUpdate::All, true, refresh_kind());
    f(&guard)
}

// ------------------------------------------------------------------- list

#[derive(Serialize)]
pub struct ProcessRow {
    pid: u32,
    parent: Option<u32>,
    name: String,
    user: String,
    /// Percent of one core (can exceed 100 for multi-threaded programs).
    cpu: f32,
    /// Resident memory: what the process occupies in RAM right now.
    memory: u64,
    /// Virtual size: the address space the process has asked the system for.
    virtual_memory: u64,
    threads: u32,
    state: &'static str,
    kernel: bool,
    run_time: u64,
}

#[derive(Serialize)]
pub struct Overview {
    processes: usize,
    running: usize,
    threads: u32,
    cpu: f32,
    cpu_count: usize,
    load: [f64; 3],
    memory_total: u64,
    memory_used: u64,
    memory_available: u64,
    swap_total: u64,
    swap_used: u64,
    uptime: u64,
}

#[derive(Serialize)]
pub struct Snapshot {
    overview: Overview,
    processes: Vec<ProcessRow>,
}

fn state_name(s: sysinfo::ProcessStatus) -> &'static str {
    use sysinfo::ProcessStatus::*;
    match s {
        Run => "Running",
        Sleep => "Sleeping",
        Idle => "Idle",
        Zombie => "Zombie",
        Stop | Tracing => "Stopped",
        UninterruptibleDiskSleep => "Disk wait",
        Dead => "Dead",
        _ => "Other",
    }
}

#[tauri::command]
pub fn process_list() -> Snapshot {
    refreshed(|st| {
        let total_threads = read("/proc/loadavg")
            .and_then(|l| l.split_whitespace().nth(3).and_then(|f| f.split('/').nth(1)?.parse::<u32>().ok()))
            .unwrap_or(0);

        let processes: Vec<ProcessRow> = st
            .sys
            .processes()
            .values()
            // Threads are reported inside their process; only list the processes.
            .filter(|p| p.thread_kind().map_or(true, |k| k == sysinfo::ThreadKind::Kernel))
            .map(|p| ProcessRow {
                pid: p.pid().as_u32(),
                parent: p.parent().map(|x| x.as_u32()),
                name: p.name().to_string_lossy().into_owned(),
                user: p
                    .user_id()
                    .and_then(|u| st.users.get_user_by_id(u))
                    .map(|u| u.name().to_owned())
                    .unwrap_or_else(|| "—".into()),
                cpu: p.cpu_usage(),
                memory: p.memory(),
                virtual_memory: p.virtual_memory(),
                threads: p.tasks().map_or(1, |t| t.len().max(1)) as u32,
                state: state_name(p.status()),
                kernel: p.thread_kind() == Some(sysinfo::ThreadKind::Kernel),
                run_time: p.run_time(),
            })
            .collect();

        let load = System::load_average();
        Snapshot {
            overview: Overview {
                processes: processes.len(),
                running: processes.iter().filter(|p| p.state == "Running").count(),
                threads: total_threads,
                cpu: st.sys.global_cpu_usage(),
                cpu_count: st.sys.cpus().len(),
                load: [load.one, load.five, load.fifteen],
                memory_total: st.sys.total_memory(),
                memory_used: st.sys.used_memory(),
                memory_available: st.sys.available_memory(),
                swap_total: st.sys.total_swap(),
                swap_used: st.sys.used_swap(),
                uptime: System::uptime(),
            },
            processes,
        }
    })
}

// ------------------------------------------------------------------ detail

#[derive(Serialize, Default)]
pub struct MemoryBreakdown {
    /// Address space the process has asked for (VmSize).
    requested: u64,
    peak_requested: u64,
    /// Physically in RAM right now (VmRSS).
    resident: u64,
    peak_resident: u64,
    anonymous: u64,
    file_backed: u64,
    shared: u64,
    swapped: u64,
    data: u64,
    stack: u64,
    code: u64,
    libraries: u64,
    page_tables: u64,
    locked: u64,
    /// Proportional share: private memory plus an equal slice of shared pages.
    proportional: Option<u64>,
    private: Option<u64>,
    shared_pages: Option<u64>,
}

#[derive(Serialize)]
pub struct Region {
    name: String,
    size: u64,
    resident: u64,
}

#[derive(Serialize)]
pub struct Child {
    pid: u32,
    name: String,
    memory: u64,
}

#[derive(Serialize)]
pub struct ProcessDetail {
    pid: u32,
    running: bool,
    /// True when the system hides this process's memory map and I/O from us.
    restricted: bool,
    memory: Option<MemoryBreakdown>,
    regions: Vec<Region>,
    children: Vec<Child>,
    details: Vec<Detail>,
}

fn kb_map(text: &str) -> HashMap<String, u64> {
    text.lines()
        .filter_map(|l| {
            let (k, v) = l.split_once(':')?;
            let n = v.trim().split_whitespace().next()?.parse::<u64>().ok()?;
            let bytes = if v.trim().ends_with("kB") { n * 1024 } else { n };
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

fn secs(ticks: u64) -> String {
    let s = ticks as f64 / 100.0; // clock ticks are 100 Hz on Linux
    if s >= 3600.0 {
        format!("{}h {}m {:.0}s", (s / 3600.0) as u64, ((s % 3600.0) / 60.0) as u64, s % 60.0)
    } else if s >= 60.0 {
        format!("{}m {:.0}s", (s / 60.0) as u64, s % 60.0)
    } else {
        format!("{s:.2} s")
    }
}

fn duration(total: u64) -> String {
    let (d, h, m) = (total / 86400, (total % 86400) / 3600, (total % 3600) / 60);
    match (d, h) {
        (0, 0) => format!("{m}m {}s", total % 60),
        (0, _) => format!("{h}h {m}m"),
        _ => format!("{d}d {h}h {m}m"),
    }
}

#[tauri::command]
pub fn process_detail(pid: u32) -> ProcessDetail {
    let dir = format!("/proc/{pid}");
    let Ok(status_text) = std::fs::read_to_string(format!("{dir}/status")) else {
        return ProcessDetail { pid, running: false, restricted: false, memory: None, regions: vec![], children: vec![], details: vec![] };
    };
    let status = kb_map(&status_text);
    let field = |k: &str| status_text.lines().find_map(|l| l.strip_prefix(&format!("{k}:"))).map(|v| v.trim().to_owned());
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
        memory.private = Some(m.get("Private_Clean").copied().unwrap_or(0) + m.get("Private_Dirty").copied().unwrap_or(0));
        memory.shared_pages = Some(m.get("Shared_Clean").copied().unwrap_or(0) + m.get("Shared_Dirty").copied().unwrap_or(0));
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
                let name = line.split_whitespace().nth(5).map(str::to_owned).unwrap_or_else(|| "(anonymous)".into());
                current = Some(name);
            } else if let Some(name) = &current {
                let entry = map.entry(name.clone()).or_default();
                if let Some(v) = line.strip_prefix("Size:") {
                    entry.0 += v.trim().trim_end_matches(" kB").parse::<u64>().unwrap_or(0) * 1024;
                } else if let Some(v) = line.strip_prefix("Rss:") {
                    entry.1 += v.trim().trim_end_matches(" kB").parse::<u64>().unwrap_or(0) * 1024;
                }
            }
        }
        regions = map.into_iter().map(|(name, (size, resident))| Region { name, size, resident }).collect();
        regions.sort_by(|a, b| b.resident.cmp(&a.resident).then(b.size.cmp(&a.size)));
        regions.truncate(10);
    }

    let stat = stat_fields(pid);
    let sf = |i: usize| stat.as_ref().and_then(|f| f.get(i)).and_then(|v| v.parse::<u64>().ok());
    // fields counted from 3 (state); index = field number - 3
    let (minflt, majflt, utime, stime) = (sf(7), sf(9), sf(11), sf(12));

    let mut d = Details::new();
    let name = field("Name").unwrap_or_default();
    d.add("Process", "Name", &name);
    d.add("Process", "ID (PID)", pid.to_string());
    d.add_opt("Process", "State", field("State"));
    d.add_opt("Process", "Threads", field("Threads"));

    let ppid: Option<u32> = field("PPid").and_then(|p| p.parse().ok());
    let (children, started, user, cmd_name_of_parent) = refreshed(|st| {
        let children: Vec<Child> = st
            .sys
            .processes()
            .values()
            .filter(|p| p.parent().map(|x| x.as_u32()) == Some(pid) && p.thread_kind().is_none())
            .map(|p| Child { pid: p.pid().as_u32(), name: p.name().to_string_lossy().into_owned(), memory: p.memory() })
            .collect();
        let me = st.sys.process(Pid::from_u32(pid));
        (
            children,
            me.map(|p| (p.start_time(), p.run_time())),
            me.and_then(|p| p.user_id()).and_then(|u| st.users.get_user_by_id(u)).map(|u| u.name().to_owned()),
            ppid.and_then(|pp| st.sys.process(Pid::from_u32(pp))).map(|p| p.name().to_string_lossy().into_owned()),
        )
    });
    d.add_opt("Process", "Owner", user);
    if let Some(pp) = ppid {
        d.add("Process", "Started by", format!("{} ({pp})", cmd_name_of_parent.unwrap_or_else(|| "?".into())));
    }
    if let Some((_, run)) = started {
        d.add("Process", "Running for", duration(run));
    }

    let cmdline = std::fs::read(format!("{dir}/cmdline"))
        .map(|b| String::from_utf8_lossy(&b).replace('\0', " ").trim().to_owned())
        .unwrap_or_default();
    d.add("Command", "Command line", if cmdline.chars().count() > 700 { format!("{}…", cmdline.chars().take(700).collect::<String>()) } else { cmdline });
    d.add_opt("Command", "Program", std::fs::read_link(format!("{dir}/exe")).ok().map(|p| p.to_string_lossy().into_owned()));
    d.add_opt("Command", "Working folder", std::fs::read_link(format!("{dir}/cwd")).ok().map(|p| p.to_string_lossy().into_owned()));
    d.add_opt(
        "Command",
        "Control group",
        read(format!("{dir}/cgroup")).and_then(|c| {
            let path = c.lines().last()?.split(':').last()?.to_owned();
            let parts: Vec<&str> = path.split('/').filter(|p| !p.is_empty()).collect();
            Some(parts[parts.len().saturating_sub(2)..].join(" / ")).filter(|s| !s.is_empty())
        }),
    );

    d.add_opt("CPU", "CPU time used", match (utime, stime) {
        (Some(u), Some(s)) => Some(format!("{} (user {} · system {})", secs(u + s), secs(u), secs(s))),
        _ => None,
    });
    d.add_opt("CPU", "Priority / nice", match (sf(15), sf(16)) {
        (Some(p), Some(n)) => Some(format!("{p} / {}", n as i64 as i32)),
        _ => None,
    });
    d.add_opt("CPU", "Last ran on core", sf(36).map(|c| c.to_string()));
    d.add_opt("CPU", "Context switches", match (field("voluntary_ctxt_switches"), field("nonvoluntary_ctxt_switches")) {
        (Some(v), Some(n)) => Some(format!("{v} voluntary · {n} forced")),
        _ => None,
    });

    d.add("Memory requests", "Asked for (virtual)", format_bytes(memory.requested));
    d.add("Memory requests", "Peak asked for", format_bytes(memory.peak_requested));
    d.add("Memory requests", "In RAM now (resident)", format_bytes(memory.resident));
    d.add("Memory requests", "Peak in RAM", format_bytes(memory.peak_resident));
    if memory.requested > 0 {
        d.add(
            "Memory requests",
            "Actually used",
            format!("{:.1}% of what it asked for", memory.resident as f64 / memory.requested as f64 * 100.0),
        );
    }
    d.add("Memory requests", "Moved to swap", format_bytes(memory.swapped));
    d.add("Memory requests", "Locked in RAM", format_bytes(memory.locked));
    d.add_opt("Page faults", "Minor (satisfied from memory)", minflt.map(|n| n.to_string()));
    d.add_opt("Page faults", "Major (had to read from disk)", majflt.map(|n| n.to_string()));
    d.add("Segments", "Heap and data", format_bytes(memory.data));
    d.add("Segments", "Stack", format_bytes(memory.stack));
    d.add("Segments", "Program code", format_bytes(memory.code));
    d.add("Segments", "Shared libraries", format_bytes(memory.libraries));
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
            l.lines().find(|x| x.starts_with("Max open files")).map(|x| x.split_whitespace().rev().nth(1).unwrap_or("?").to_owned())
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

    ProcessDetail { pid, running: true, restricted, memory: Some(memory), regions, children, details: d.finish() }
}
