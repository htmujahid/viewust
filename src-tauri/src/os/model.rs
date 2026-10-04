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
