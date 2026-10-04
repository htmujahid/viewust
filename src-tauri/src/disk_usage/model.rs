use serde::Serialize;

/// A partition, logical volume or encrypted layer on a disk, or a network share.
#[derive(Serialize, Clone, Debug, PartialEq)]
pub struct Volume {
    /// `/dev/sda2`, or the mount point for a share. Unique across the list.
    pub(crate) path: String,
    pub(crate) name: String,
    pub(crate) size: u64,
    pub(crate) fstype: Option<String>,
    pub(crate) label: Option<String>,
    /// Where it is mounted, if it is
    pub(crate) mount: Option<String>,
    pub(crate) used: Option<u64>,
    pub(crate) available: Option<u64>,
    /// "filesystem", "swap", "container" (holds other volumes) or "empty" (no filesystem)
    pub(crate) role: &'static str,
    /// A filesystem that isn't mounted, so it can be mounted to browse it
    pub(crate) mountable: bool,
    pub(crate) children: Vec<Volume>,
}

/// A physical drive, or the group of network shares.
#[derive(Serialize, Clone, Debug, PartialEq)]
pub struct Disk {
    pub(crate) path: String,
    pub(crate) name: String,
    pub(crate) model: Option<String>,
    pub(crate) size: u64,
    /// "nvme", "ssd", "hdd", "usb", "optical" or "network"
    pub(crate) kind: &'static str,
    pub(crate) removable: bool,
    pub(crate) volumes: Vec<Volume>,
}

#[derive(Serialize, Default, PartialEq, Debug)]
pub struct Overview {
    /// Combined size of the physical drives
    pub(crate) capacity: u64,
    pub(crate) used: u64,
    pub(crate) available: u64,
    pub(crate) disks: usize,
    pub(crate) mounted: usize,
    pub(crate) unmounted: usize,
}

#[derive(Serialize)]
pub struct Devices {
    pub(crate) overview: Overview,
    pub(crate) disks: Vec<Disk>,
}

#[derive(Serialize, Debug)]
pub struct UsageEntry {
    pub(crate) name: String,
    pub(crate) path: String,
    /// "dir", "file", "link" or "other"
    pub(crate) kind: &'static str,
    pub(crate) size: u64,
    /// Files inside, for a folder
    pub(crate) files: u64,
    pub(crate) unreadable: bool,
    /// A folder that is another filesystem mounted here, so it is not counted in its parent
    pub(crate) mount: bool,
}

#[derive(Serialize, Debug)]
pub struct DirectoryUsage {
    pub(crate) path: String,
    pub(crate) total: u64,
    pub(crate) entries: Vec<UsageEntry>,
    pub(crate) hidden_count: usize,
    pub(crate) hidden_size: u64,
    pub(crate) unreadable: bool,
    pub(crate) incomplete: bool,
    pub(crate) took_ms: u64,
}
