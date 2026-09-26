//! NVIDIA's `nvidia-smi`, the only way to read these cards' live state without the driver SDK.

use super::cmd::run;

pub(crate) struct Smi {
    pub(crate) values: Vec<String>,
}

pub(crate) fn nvidia_smi() -> Vec<Smi> {
    run(
        "nvidia-smi",
        &[
            "--query-gpu=pci.bus_id,name,memory.total,memory.used,temperature.gpu,power.draw,power.limit,\
             clocks.gr,clocks.max.gr,clocks.mem,driver_version,vbios_version,fan.speed,utilization.gpu,\
             pcie.link.gen.current,pcie.link.gen.max,pcie.link.width.current,pcie.link.width.max",
            "--format=csv,noheader,nounits",
        ],
    )
    .map(|t| {
        t.lines()
            .map(|l| Smi { values: l.split(", ").map(|v| v.trim().to_owned()).collect() })
            .collect()
    })
    .unwrap_or_default()
}
