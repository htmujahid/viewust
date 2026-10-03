use std::sync::{Arc, Mutex};
use std::time::Duration;

use sysinfo::{ProcessRefreshKind, ProcessesToUpdate, System, UpdateKind, Users};

pub(super) struct Tables {
    pub(super) sys: System,
    pub(super) users: Users,
}

fn refresh_kind() -> ProcessRefreshKind {
    ProcessRefreshKind::nothing()
        .with_cpu()
        .with_memory()
        .with_user(UpdateKind::OnlyIfNotSet)
        .with_tasks()
}

impl Tables {
    fn new() -> Self {
        let mut sys = System::new();
        sys.refresh_cpu_all();
        sys.refresh_processes_specifics(ProcessesToUpdate::All, true, refresh_kind());
        std::thread::sleep(Duration::from_millis(300));
        Self {
            sys,
            users: Users::new_with_refreshed_list(),
        }
    }
}

#[derive(Clone, Default)]
pub struct ProcessService(Arc<Mutex<Option<Tables>>>);

impl ProcessService {
    pub(super) fn with_fresh<R>(&self, read: impl FnOnce(&Tables) -> R) -> R {
        let mut guard = self.0.lock().unwrap_or_else(|e| e.into_inner());
        let tables = guard.get_or_insert_with(Tables::new);
        tables.sys.refresh_cpu_all();
        tables.sys.refresh_memory();
        tables
            .sys
            .refresh_processes_specifics(ProcessesToUpdate::All, true, refresh_kind());
        read(tables)
    }
}
