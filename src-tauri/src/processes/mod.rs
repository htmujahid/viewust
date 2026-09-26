//! Running processes and how each one uses memory.
//!
//! The list comes from `sysinfo`, kept alive between calls so CPU usage can be
//! measured over time. The detail view reads `/proc/<pid>` directly. A process
//! owned by another user exposes only basic facts, so everything beyond that is
//! optional and the result says when it was restricted.

pub mod commands;
mod detail;
mod list;
mod model;
mod service;

pub use model::{ProcessDetail, Snapshot};
pub use service::ProcessService;
