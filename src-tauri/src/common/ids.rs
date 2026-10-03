pub(crate) fn lookup_ids(
    files: &[&str],
    vendor: &str,
    device: Option<&str>,
) -> (Option<String>, Option<String>) {
    let Some(text) = files.iter().find_map(|f| std::fs::read_to_string(f).ok()) else {
        return (None, None);
    };
    lookup_text(&text, vendor, device)
}

pub(crate) fn lookup_text(
    text: &str,
    vendor: &str,
    device: Option<&str>,
) -> (Option<String>, Option<String>) {
    let (vendor, device) = (vendor.to_lowercase(), device.map(str::to_lowercase));
    let mut vendor_name = None;
    let mut device_name = None;
    for line in text.lines() {
        if line.starts_with('#') || line.trim().is_empty() {
            continue;
        }
        if !line.starts_with('\t') {
            if vendor_name.is_some() {
                break;
            }
            if let Some((id, name)) = line.split_once("  ") {
                if id.to_lowercase() == vendor {
                    vendor_name = Some(name.trim().to_owned());
                }
            }
        } else if vendor_name.is_some() && !line.starts_with("\t\t") {
            if let (Some(want), Some((id, name))) = (&device, line.trim_start().split_once("  ")) {
                if id.to_lowercase() == *want {
                    device_name = Some(name.trim().to_owned());
                    break;
                }
            }
        }
    }
    (vendor_name, device_name)
}

pub(crate) const USB_IDS: &[&str] = &[
    "/usr/share/hwdata/usb.ids",
    "/usr/share/misc/usb.ids",
    "/var/lib/usbutils/usb.ids",
];
pub(crate) const PCI_IDS: &[&str] = &["/usr/share/hwdata/pci.ids", "/usr/share/misc/pci.ids"];

pub fn pnp_vendor(code: &str) -> Option<String> {
    let text = std::fs::read_to_string("/usr/share/hwdata/pnp.ids").ok()?;
    text.lines().find_map(|l| {
        let (id, name) = l.split_once('\t')?;
        (id == code).then(|| name.trim().to_owned())
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const DB: &str = "# comment\n\
2717  Xiaomi Inc.\n\
\t0011  100Mbps Network Card Adapter\n\
\t0360  Mi3W\n\
\t\t0001  a sub-entry that must be ignored\n\
046d  Logitech, Inc.\n\
\t082d  HD Pro Webcam C920\n";

    #[test]
    fn finds_vendor_and_device() {
        assert_eq!(
            lookup_text(DB, "2717", Some("0360")),
            (Some("Xiaomi Inc.".into()), Some("Mi3W".into()))
        );
        assert_eq!(
            lookup_text(DB, "046D", Some("082D")),
            (
                Some("Logitech, Inc.".into()),
                Some("HD Pro Webcam C920".into())
            )
        );
    }

    #[test]
    fn unknown_ids_give_partial_answers() {
        assert_eq!(lookup_text(DB, "ffff", Some("0001")), (None, None));
        assert_eq!(
            lookup_text(DB, "2717", Some("9999")),
            (Some("Xiaomi Inc.".into()), None)
        );
        assert_eq!(
            lookup_text(DB, "046d", None),
            (Some("Logitech, Inc.".into()), None)
        );
    }

    #[test]
    fn does_not_leak_into_the_next_vendor() {
        assert_eq!(lookup_text(DB, "2717", Some("082d")).1, None);
    }
}
