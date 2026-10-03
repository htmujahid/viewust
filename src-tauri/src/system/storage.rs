use super::model::*;
use crate::common::cmd::run;
use crate::common::format::*;
use crate::common::sysfs::*;
use crate::common::Details;

pub(crate) fn storage() -> Vec<Component> {
    let Some(json) = run(
        "lsblk",
        &["-J", "-b", "-o", "NAME,SIZE,TYPE,TRAN,ROTA,MODEL,SERIAL,REV,VENDOR,WWN,MOUNTPOINTS,FSTYPE,LABEL,HOTPLUG,RM"],
    )
    .and_then(|t| serde_json::from_str::<serde_json::Value>(&t).ok()) else {
        return Vec::new();
    };

    let text = |v: &serde_json::Value, k: &str| {
        v[k].as_str()
            .map(|s| s.trim().to_owned())
            .filter(|s| !s.is_empty())
    };
    let mut out = Vec::new();

    for disk in json["blockdevices"].as_array().into_iter().flatten() {
        let name = text(disk, "name").unwrap_or_default();
        let tran = text(disk, "tran").unwrap_or_default();
        if disk["type"] != "disk" || disk["hotplug"] == true || disk["rm"] == true || tran == "usb"
        {
            continue;
        }
        let size = disk["size"].as_u64().unwrap_or(0);
        let is_nvme = name.starts_with("nvme") || tran == "nvme";
        let spinning = disk["rota"] == true;
        let kind = if is_nvme {
            "nvme"
        } else if spinning {
            "hdd"
        } else {
            "ssd"
        };
        let interface = match tran.as_str() {
            "sata" | "ata" => "SATA",
            "nvme" => "NVMe",
            "sas" => "SAS",
            "" if is_nvme => "NVMe",
            "" => "Internal",
            other => other,
        };

        let udev = run(
            "udevadm",
            &["info", "--query=property", &format!("--name=/dev/{name}")],
        )
        .unwrap_or_default();
        let prop = |k: &str| {
            udev.lines()
                .find_map(|l| l.strip_prefix(&format!("{k}=")).map(str::to_owned))
        };
        let model = prop("ID_MODEL")
            .map(|m| m.replace('_', " "))
            .or_else(|| text(disk, "model"))
            .unwrap_or_else(|| name.clone());
        let q = |f: &str| read(format!("/sys/block/{name}/queue/{f}"));

        let mut d = Details::new();
        d.add("Drive", "Model", &model);
        d.add_opt("Drive", "Vendor", text(disk, "vendor"));
        d.add_opt(
            "Drive",
            "Serial number",
            prop("ID_SERIAL_SHORT").or_else(|| text(disk, "serial")),
        );
        d.add_opt(
            "Drive",
            "Firmware",
            prop("ID_REVISION").or_else(|| text(disk, "rev")),
        );
        d.add_opt("Drive", "WWN", prop("ID_WWN").or_else(|| text(disk, "wwn")));
        d.add(
            "Capacity",
            "Size",
            format!("{} ({})", decimal_size(size), format_bytes(size)),
        );
        d.add(
            "Type",
            "Technology",
            match (kind, prop("ID_ATA_ROTATION_RATE_RPM")) {
                ("hdd", Some(rpm)) if rpm != "0" => format!("Hard disk · {rpm} rpm"),
                ("hdd", _) => "Hard disk".to_owned(),
                ("nvme", _) => "Solid-state drive (NVMe)".to_owned(),
                _ => "Solid-state drive".to_owned(),
            },
        );
        d.add("Type", "Interface", interface);
        d.add("Type", "Device", format!("/dev/{name}"));
        d.add_opt(
            "Type",
            "Sector size",
            match (q("logical_block_size"), q("physical_block_size")) {
                (Some(l), Some(p)) => Some(format!("{l} B logical · {p} B physical")),
                _ => None,
            },
        );
        let mut features = Vec::new();
        for (key, label) in [
            (
                "ID_ATA_FEATURE_SET_SMART_ENABLED",
                "SMART health monitoring",
            ),
            ("ID_ATA_WRITE_CACHE_ENABLED", "Write cache"),
            ("ID_ATA_FEATURE_SET_APM_ENABLED", "Power management (APM)"),
            ("ID_ATA_FEATURE_SET_PM_ENABLED", "Power saving"),
        ] {
            if prop(key).as_deref() == Some("1") {
                features.push(label);
            }
        }
        if !features.is_empty() {
            d.add("Type", "Enabled features", features.join(", "));
        }
        if is_nvme {
            d.add_opt(
                "Health",
                "Temperature",
                hwmon_temp(&["nvme"], None).map(|t| format!("{t:.0} °C")),
            );
        }

        for part in disk["children"].as_array().into_iter().flatten() {
            let mounts: Vec<String> = part["mountpoints"]
                .as_array()
                .into_iter()
                .flatten()
                .filter_map(|m| m.as_str().map(str::to_owned))
                .collect();
            d.add(
                "Partitions",
                text(part, "name").unwrap_or_default(),
                [
                    Some(decimal_size(part["size"].as_u64().unwrap_or(0))),
                    text(part, "fstype"),
                    text(part, "label"),
                    (!mounts.is_empty()).then(|| format!("mounted at {}", mounts.join(", "))),
                ]
                .into_iter()
                .flatten()
                .collect::<Vec<_>>()
                .join(" · "),
            );
        }

        out.push(Component {
            id: format!("sys:disk:{name}"),
            kind,
            name: model,
            subtitle: Some(format!("{} · {interface}", decimal_size(size))),
            details: d.finish(),
        });
    }
    out
}
