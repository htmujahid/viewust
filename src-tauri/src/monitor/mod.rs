//! One-second live samples of CPU, memory, drives, graphics cards and network.

pub mod commands;
mod model;
mod service;
mod sources;

pub use model::Sample;
pub use service::MonitorService;
