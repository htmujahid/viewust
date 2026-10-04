use super::driver::driver_rows;
use crate::common::ids::*;
use crate::common::sysfs::*;
use crate::common::Details;
use std::path::Path;

pub(crate) fn pci_rows(d: &mut Details, section: &str, device_dir: &Path) {
    let hex = |f: &str| read(device_dir.join(f)).map(|v| v.trim_start_matches("0x").to_owned());
    let (vendor, device) = (hex("vendor"), hex("device"));
    if let (Some(v), Some(dv)) = (&vendor, &device) {
        let (vname, dname) = lookup_ids(PCI_IDS, v, Some(dv));
        d.add_opt(section, "Vendor", vname);
        d.add_opt(section, "Model", dname);
        d.add(section, "PCI ID", format!("{v}:{dv}"));
    }
    if let Some(slot) = std::fs::canonicalize(device_dir)
        .ok()
        .and_then(|p| p.file_name().map(|n| n.to_string_lossy().into_owned()))
    {
        d.add(section, "PCI address", slot);
    }
    let driver = std::fs::read_link(device_dir.join("driver"))
        .ok()
        .and_then(|p| p.file_name().map(|f| f.to_string_lossy().into_owned()));
    if let Some(driver) = driver {
        driver_rows(d, &format!("{section} · driver"), &driver);
    }
}
