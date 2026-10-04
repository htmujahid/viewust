use serde::Serialize;

use crate::common::Detail;

#[derive(Serialize, Clone)]
pub struct FilesystemRow {
    pub(crate) mount: String,
    pub(crate) source: String,
    pub(crate) fstype: String,
    pub(crate) kind: String,
    pub(crate) read_only: bool,
    pub(crate) size: Option<u64>,
    pub(crate) used: Option<u64>,
    pub(crate) available: Option<u64>,
    pub(crate) inodes_total: Option<u64>,
    pub(crate) inodes_used: Option<u64>,
    pub(crate) label: Option<String>,
    pub(crate) details: Vec<Detail>,
}

#[derive(Serialize, Default, PartialEq, Debug)]
pub struct Overview {
    pub(crate) size: u64,
    pub(crate) used: u64,
    pub(crate) available: u64,
    pub(crate) volumes: usize,
}

#[derive(Serialize)]
pub struct Snapshot {
    pub(crate) overview: Overview,
    pub(crate) filesystems: Vec<FilesystemRow>,
}
