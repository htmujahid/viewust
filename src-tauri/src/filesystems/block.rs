use std::collections::HashMap;

pub(crate) struct Block {
    pub(crate) name: String,
    pub(crate) parent: Option<String>,
    pub(crate) kind: String,
    pub(crate) model: Option<String>,
    pub(crate) transport: Option<String>,
    pub(crate) removable: bool,
    pub(crate) rotational: bool,
    pub(crate) uuid: Option<String>,
    pub(crate) label: Option<String>,
}

pub(crate) const LSBLK_COLUMNS: &str =
    "NAME,KNAME,PKNAME,TYPE,MODEL,TRAN,HOTPLUG,RM,ROTA,UUID,LABEL";

fn text(v: &serde_json::Value, key: &str) -> Option<String> {
    v[key]
        .as_str()
        .map(|s| s.trim().to_owned())
        .filter(|s| !s.is_empty())
}

/// Older lsblk prints flags as "1"/"0", newer as true/false.
fn flag(v: &serde_json::Value, key: &str) -> bool {
    matches!(&v[key], serde_json::Value::Bool(true))
        || matches!(v[key].as_str(), Some("1" | "true"))
}

/// Keyed by kernel name (`dm-0`, `sda1`), which is what `/dev/mapper/x` resolves to.
pub(crate) fn parse_lsblk(json: &str) -> HashMap<String, Block> {
    let Ok(root) = serde_json::from_str::<serde_json::Value>(json) else {
        return HashMap::new();
    };
    let mut out = HashMap::new();
    for dev in root["blockdevices"].as_array().into_iter().flatten() {
        let Some(key) = text(dev, "kname").or_else(|| text(dev, "name")) else {
            continue;
        };
        out.entry(key.clone()).or_insert_with(|| Block {
            name: text(dev, "name").unwrap_or(key),
            parent: text(dev, "pkname"),
            kind: text(dev, "type").unwrap_or_default(),
            model: text(dev, "model"),
            transport: text(dev, "tran"),
            removable: flag(dev, "hotplug") || flag(dev, "rm"),
            rotational: flag(dev, "rota"),
            uuid: text(dev, "uuid"),
            label: text(dev, "label"),
        });
    }
    out
}

/// The device and everything beneath it, outwards: `dm-0` → its partition → its disk.
pub(crate) fn stack<'a>(blocks: &'a HashMap<String, Block>, kname: &str) -> Vec<&'a Block> {
    let mut out = Vec::new();
    let mut next = Some(kname.to_owned());
    while let Some(name) = next {
        let Some(block) = blocks.get(&name) else {
            break;
        };
        if out.len() >= 8 {
            break;
        }
        out.push(block);
        next = block.parent.clone();
    }
    out
}

pub(crate) fn is_removable(stack: &[&Block]) -> bool {
    stack
        .iter()
        .any(|b| b.removable || b.transport.as_deref() == Some("usb"))
}

#[cfg(test)]
mod tests {
    use super::*;

    const LSBLK: &str = r#"{"blockdevices":[
      {"name":"nvme0n1","kname":"nvme0n1","pkname":null,"type":"disk","model":"Samsung SSD 980 ","tran":"nvme","hotplug":false,"rm":false,"rota":false,"uuid":null,"label":null},
      {"name":"nvme0n1p3","kname":"nvme0n1p3","pkname":"nvme0n1","type":"part","model":null,"tran":null,"hotplug":false,"rm":false,"rota":false,"uuid":"abc","label":null},
      {"name":"vg-root","kname":"dm-0","pkname":"nvme0n1p3","type":"lvm","model":null,"tran":null,"hotplug":false,"rm":false,"rota":false,"uuid":"def","label":"system"},
      {"name":"sdb","kname":"sdb","pkname":null,"type":"disk","model":"Flash","tran":"usb","hotplug":"1","rm":"1","rota":"0","uuid":null,"label":null},
      {"name":"sdb1","kname":"sdb1","pkname":"sdb","type":"part","model":null,"tran":"usb","hotplug":"1","rm":"1","rota":"0","uuid":"1234","label":"STICK"}]}"#;

    #[test]
    fn devices_are_keyed_by_kernel_name_and_keep_their_labels() {
        let b = parse_lsblk(LSBLK);
        assert_eq!(b.len(), 5);
        assert_eq!(b["dm-0"].name, "vg-root");
        assert_eq!(b["dm-0"].label.as_deref(), Some("system"));
        assert_eq!(b["nvme0n1"].model.as_deref(), Some("Samsung SSD 980"));
    }

    #[test]
    fn the_stack_walks_from_the_volume_down_to_the_disk() {
        let b = parse_lsblk(LSBLK);
        let names: Vec<&str> = stack(&b, "dm-0").iter().map(|x| x.kind.as_str()).collect();
        assert_eq!(names, ["lvm", "part", "disk"]);
        assert!(stack(&b, "nope").is_empty());
    }

    #[test]
    fn a_usb_stick_is_removable_and_an_nvme_drive_is_not() {
        let b = parse_lsblk(LSBLK);
        assert!(is_removable(&stack(&b, "sdb1")));
        assert!(!is_removable(&stack(&b, "dm-0")));
    }

    #[test]
    fn bad_json_gives_nothing() {
        assert!(parse_lsblk("not json").is_empty());
    }
}
