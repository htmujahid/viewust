//! Facts about the computer itself, for the sidebar.

use crate::common::format::*;
use crate::common::sysfs::*;
use crate::common::{Detail, Details};
use sysinfo::System;

pub(crate) fn details() -> Vec<Detail> {
    let mut sys = System::new();
    sys.refresh_memory();
    sys.refresh_cpu_all();

    let mut d = Details::new();

    #[cfg(target_os = "linux")]
    {
        let dmi = |f: &str| read(format!("/sys/class/dmi/id/{f}"));
        d.add_opt("System", "Manufacturer", dmi("sys_vendor"));
        d.add_opt("System", "Model", dmi("product_name"));
        d.add_opt("System", "Motherboard", dmi("board_name"));
        d.add_opt(
            "System",
            "BIOS",
            dmi("bios_vendor").map(|v| match dmi("bios_version") {
                Some(ver) => format!("{v} {ver}"),
                None => v,
            }),
        );
    }

    d.add_opt("Operating system", "Host name", System::host_name());
    d.add_opt("Operating system", "System", System::long_os_version());
    d.add_opt("Operating system", "Kernel", System::kernel_version());
    d.add("Operating system", "Architecture", System::cpu_arch());
    let up = System::uptime();
    d.add(
        "Operating system",
        "Uptime",
        format!(
            "{}d {}h {}m",
            up / 86400,
            (up % 86400) / 3600,
            (up % 3600) / 60
        ),
    );

    if let Some(cpu) = sys.cpus().first() {
        d.add("Processor", "Model", cpu.brand().trim());
        d.add("Processor", "Vendor", cpu.vendor_id());
    }
    d.add_opt(
        "Processor",
        "Physical cores",
        System::physical_core_count().map(|n| n.to_string()),
    );
    d.add("Processor", "Logical cores", sys.cpus().len().to_string());

    d.add("Memory", "Installed", format_bytes(sys.total_memory()));
    d.add("Memory", "In use", format_bytes(sys.used_memory()));
    if sys.total_swap() > 0 {
        d.add("Memory", "Swap", format_bytes(sys.total_swap()));
    }
    d.finish()
}
