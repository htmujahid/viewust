use super::model::*;
use super::service::ProcessService;
use crate::common::sysfs::*;
use sysinfo::System;

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

impl ProcessService {
    pub fn snapshot(&self) -> Snapshot {
        self.with_fresh(|st| {
            let total_threads = read("/proc/loadavg")
                .and_then(|l| {
                    l.split_whitespace()
                        .nth(3)
                        .and_then(|f| f.split('/').nth(1)?.parse::<u32>().ok())
                })
                .unwrap_or(0);

            let processes: Vec<ProcessRow> = st
                .sys
                .processes()
                .values()
                .filter(|p| {
                    p.thread_kind()
                        .map_or(true, |k| k == sysinfo::ThreadKind::Kernel)
                })
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
}
