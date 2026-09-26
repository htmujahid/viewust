//! The computer's internals: motherboard, processor, memory, drives, graphics
//! cards, network and sound adapters, and what is known about power.
//!
//! Everything is read without administrator rights. The two things the
//! operating system won't hand over freely are called out in the results:
//! individual memory modules (see [`memory::read_modules`]) and the power
//! supply, which never reports itself.

mod board;
pub mod commands;
mod cpu;
pub mod deep;
mod gpu;
pub mod memory;
mod model;
mod network;
mod pci;
mod power;
mod sound;
mod storage;

pub use model::{MemoryModules, SystemInfo};

/// Gathers every internal component.
pub fn collect() -> SystemInfo {
    let pci = pci::Pci::load();
    let mut components = vec![board::board(&pci), cpu::cpu(), memory::memory()];
    components.extend(gpu::gpus(&pci));
    components.extend(storage::storage());
    components.extend(network::network(&pci));
    components.extend(sound::sound(&pci));
    components.push(power::power());
    SystemInfo {
        computer_name: sysinfo::System::host_name().unwrap_or_else(|| "This computer".into()),
        components,
    }
}
