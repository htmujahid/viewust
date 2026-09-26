//! Infrastructure shared by every feature. Nothing in here knows about devices.

pub mod blocking;
pub mod cmd;
pub mod details;
pub mod format;
pub mod ids;
pub mod nvidia;
pub mod sysfs;

pub use details::{Detail, Details};
