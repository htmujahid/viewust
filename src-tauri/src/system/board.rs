use super::model::*;
use super::pci::{pci_model, Pci};
use crate::common::sysfs::*;
use crate::common::Details;
use std::path::Path;

fn chassis_name(code: &str) -> &'static str {
    match code.parse::<u32>().unwrap_or(0) {
        3 => "Desktop",
        4 => "Low-profile desktop",
        5 => "Pizza box",
        6 => "Mini tower",
        7 => "Tower",
        8 => "Portable",
        9 => "Laptop",
        10 => "Notebook",
        13 => "All-in-one",
        15 => "Space-saving",
        16 => "Lunch box",
        17 => "Server",
        23 => "Rack server",
        30 => "Tablet",
        31 => "Convertible",
        35 => "Mini PC",
        36 => "Stick PC",
        _ => "Other",
    }
}

pub(crate) fn board(pci: &Pci) -> Component {
    let mut d = Details::new();
    d.add_opt("Motherboard", "Manufacturer", dmi("board_vendor"));
    d.add_opt("Motherboard", "Model", dmi("board_name"));
    d.add_opt("Motherboard", "Revision", dmi("board_version"));
    d.add_opt("System", "Manufacturer", dmi("sys_vendor"));
    d.add_opt("System", "Model", dmi("product_name"));
    d.add_opt("System", "Family", dmi("product_family"));
    d.add_opt(
        "System",
        "Case type",
        read("/sys/class/dmi/id/chassis_type").map(|c| chassis_name(&c)),
    );
    d.add_opt("BIOS / UEFI", "Vendor", dmi("bios_vendor"));
    d.add_opt("BIOS / UEFI", "Version", dmi("bios_version"));
    d.add_opt("BIOS / UEFI", "Date", dmi("bios_date"));
    d.add_opt("BIOS / UEFI", "Release", dmi("bios_release"));
    d.add(
        "BIOS / UEFI",
        "Boot mode",
        if Path::new("/sys/firmware/efi").exists() {
            "UEFI"
        } else {
            "Legacy BIOS"
        },
    );
    if let Some(c) = pci.by_class(0x0601) {
        d.add("Chipset", "Chipset", pci_model(c));
        d.add_opt("Chipset", "Vendor", c.vendor_name.clone());
    }
    if let Some(h) = pci.by_class(0x0600) {
        d.add("Chipset", "Host bridge", pci_model(h));
    }
    d.add(
        "Chipset",
        "PCI devices",
        format!("{} on the board (see technical details)", pci.0.len()),
    );
    Component {
        id: "sys:board".into(),
        kind: "board",
        name: dmi("board_name").unwrap_or_else(|| "Motherboard".into()),
        subtitle: dmi("board_vendor"),
        details: d.finish(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn case_types_have_names() {
        assert_eq!(chassis_name("3"), "Desktop");
        assert_eq!(chassis_name("9"), "Laptop");
        assert_eq!(chassis_name("999"), "Other");
        assert_eq!(chassis_name("not a number"), "Other");
    }
}
