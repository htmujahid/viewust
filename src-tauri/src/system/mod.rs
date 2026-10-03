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
