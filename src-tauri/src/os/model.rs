use serde::Serialize;

use crate::common::Detail;

#[derive(Serialize)]
pub struct OsSummary {
    pub(crate) name: String,
    pub(crate) version: Option<String>,
    pub(crate) kernel: String,
    pub(crate) hostname: String,
    pub(crate) architecture: String,
    /// Seconds since the epoch
    pub(crate) boot_time: u64,
    pub(crate) details: Vec<Detail>,
}

#[derive(Serialize, Debug, PartialEq)]
pub struct CgroupRow {
    pub(crate) name: String,
    /// Processes inside it and its children (`pids.current`)
    pub(crate) pids: Option<u64>,
    /// Memory charged to it (`memory.current`)
    pub(crate) memory: Option<u64>,
    /// Groups nested beneath it (`cgroup.stat`)
    pub(crate) groups: Option<u64>,
}

#[derive(Serialize)]
pub struct Cgroups {
    pub(crate) version: &'static str,
    pub(crate) controllers: Vec<String>,
    pub(crate) groups: usize,
    /// The count hit its ceiling, so the real number is higher
    pub(crate) capped: bool,
    /// The groups directly under the root, biggest first
    pub(crate) top: Vec<CgroupRow>,
}

#[derive(Serialize)]
pub struct NetInterface {
    pub(crate) name: String,
    /// "ethernet", "wifi", "bridge", "virtual" or "loopback"
    pub(crate) kind: &'static str,
    pub(crate) state: String,
    pub(crate) mac: Option<String>,
    pub(crate) mtu: Option<u64>,
    pub(crate) speed: Option<String>,
    pub(crate) ipv4: Vec<String>,
    pub(crate) ipv6: usize,
    pub(crate) rx: u64,
    pub(crate) tx: u64,
}

#[derive(Serialize)]
pub struct OsNetwork {
    /// Real interfaces that are up, and how many exist (the loopback doesn't count)
    pub(crate) up: usize,
    pub(crate) total: usize,
    pub(crate) default_route: Option<String>,
    pub(crate) dns: Vec<String>,
    pub(crate) listening_tcp: Vec<u16>,
    pub(crate) established: usize,
    pub(crate) interfaces: Vec<NetInterface>,
    pub(crate) details: Vec<Detail>,
}

#[derive(Serialize, Debug, PartialEq)]
pub struct ConnRow {
    pub(crate) proto: &'static str,
    pub(crate) local: String,
    pub(crate) remote: String,
    pub(crate) state: &'static str,
}

#[derive(Serialize)]
pub struct Connections {
    pub(crate) established: usize,
    pub(crate) listening: usize,
    pub(crate) time_wait: usize,
    pub(crate) rows: Vec<ConnRow>,
}

#[derive(Serialize)]
pub struct OsLogs {
    pub(crate) available: bool,
    pub(crate) size: Option<String>,
    pub(crate) boots: Option<usize>,
    /// The error-level lines of this boot, oldest first
    pub(crate) errors: Vec<String>,
    pub(crate) truncated: bool,
    pub(crate) note: Option<String>,
}

#[derive(Serialize)]
pub struct LoginSession {
    pub(crate) id: String,
    pub(crate) user: String,
    /// "wayland", "x11" or "tty"
    pub(crate) kind: String,
    pub(crate) class: String,
    /// The TTY, seat or remote host it comes from
    pub(crate) place: String,
    pub(crate) remote: bool,
    pub(crate) since: Option<String>,
    pub(crate) state: String,
}

#[derive(Serialize)]
pub struct OsLogins {
    pub(crate) available: bool,
    pub(crate) sessions: Vec<LoginSession>,
}

#[derive(Serialize)]
pub struct OsMemory {
    pub(crate) total: u64,
    pub(crate) used: u64,
    pub(crate) available: u64,
    pub(crate) swap_total: u64,
    pub(crate) swap_used: u64,
    pub(crate) details: Vec<Detail>,
}

#[derive(Serialize)]
pub struct OsSecurity {
    pub(crate) apparmor: Option<String>,
    pub(crate) selinux: Option<String>,
    pub(crate) lockdown: Option<String>,
    pub(crate) secure_boot: Option<String>,
    pub(crate) details: Vec<Detail>,
}

#[derive(Serialize, Debug, PartialEq)]
pub struct KernelModule {
    pub(crate) name: String,
    pub(crate) size: u64,
    pub(crate) used_by: Vec<String>,
}

#[derive(Serialize)]
pub struct ModuleInfo {
    pub(crate) name: String,
    pub(crate) found: bool,
    pub(crate) details: Vec<Detail>,
}

#[derive(Serialize, Debug, PartialEq)]
pub struct EnvVar {
    pub(crate) key: String,
    /// Empty when `hidden`
    pub(crate) value: String,
    /// Looks like a secret, so the value is not sent to the window at all
    pub(crate) hidden: bool,
}

#[derive(Serialize, Debug, PartialEq)]
pub struct PackageRow {
    pub(crate) name: String,
    pub(crate) version: String,
    pub(crate) arch: Option<String>,
}

#[derive(Serialize)]
pub struct Packages {
    /// "apt (dpkg)", "rpm" or "pacman"; none when no known package manager is installed
    pub(crate) manager: Option<&'static str>,
    pub(crate) packages: Vec<PackageRow>,
}
