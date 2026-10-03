use crate::common::ids::*;
use crate::common::sysfs::*;
use std::path::Path;

pub(crate) struct PciDevice {
    pub(crate) slot: String,
    pub(crate) class: u32,
    pub(crate) vendor: String,
    pub(crate) device: String,
    pub(crate) vendor_name: Option<String>,
    pub(crate) device_name: Option<String>,
}

pub(crate) struct Pci(pub(crate) Vec<PciDevice>);

impl Pci {
    pub(crate) fn load() -> Self {
        let text = PCI_IDS
            .iter()
            .find_map(|f| std::fs::read_to_string(f).ok())
            .unwrap_or_default();
        let hex =
            |p: &Path, f: &str| read(p.join(f)).map(|v| v.trim_start_matches("0x").to_owned());
        let mut out: Vec<PciDevice> = std::fs::read_dir("/sys/bus/pci/devices")
            .into_iter()
            .flatten()
            .flatten()
            .filter_map(|e| {
                let p = e.path();
                let (vendor, device) = (hex(&p, "vendor")?, hex(&p, "device")?);
                let (vendor_name, device_name) = lookup_text(&text, &vendor, Some(&device));
                Some(PciDevice {
                    slot: e.file_name().to_string_lossy().into_owned(),
                    class: u32::from_str_radix(&hex(&p, "class")?, 16).ok()?,
                    vendor,
                    device,
                    vendor_name,
                    device_name,
                })
            })
            .collect();
        out.sort_by(|a, b| a.slot.cmp(&b.slot));
        Self(out)
    }

    pub(crate) fn at(&self, slot: &str) -> Option<&PciDevice> {
        self.0.iter().find(|d| d.slot == slot)
    }

    pub(crate) fn by_class(&self, class: u32) -> Option<&PciDevice> {
        self.0.iter().find(|d| d.class >> 8 == class)
    }
}

pub(crate) fn pci_class_name(class: u32) -> &'static str {
    match class >> 8 {
        0x0100 => "SCSI controller",
        0x0101 => "IDE controller",
        0x0106 => "SATA controller",
        0x0108 => "NVMe controller",
        0x0200 => "Ethernet controller",
        0x0280 => "Network controller",
        0x0300 => "VGA controller",
        0x0302 => "3D controller",
        0x0401 => "Audio device",
        0x0403 => "Audio controller",
        0x0500 => "RAM controller",
        0x0600 => "Host bridge",
        0x0601 => "ISA/LPC bridge",
        0x0604 => "PCI bridge",
        0x0c03 => "USB controller",
        0x0c05 => "SMBus controller",
        0x0780 => "Communication controller",
        0x0880 => "System peripheral",
        0x1180 => "Signal processing controller",
        _ => "Other device",
    }
}

pub(crate) fn pci_model(d: &PciDevice) -> String {
    d.device_name
        .clone()
        .unwrap_or_else(|| format!("{}:{}", d.vendor, d.device))
}
