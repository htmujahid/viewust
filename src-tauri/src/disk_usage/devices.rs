use std::collections::{HashMap, HashSet};

use super::model::{Devices, Disk, Overview, Volume};
use super::mounts::{parse_mountinfo, visible};
use super::usage::{parse_df, Usage, DF_COLUMNS};
use crate::common::cmd::run;
use crate::error::{AppError, Result};

pub(crate) const LSBLK_COLUMNS: &str =
    "NAME,PATH,TYPE,SIZE,MODEL,TRAN,ROTA,RM,HOTPLUG,FSTYPE,LABEL,MOUNTPOINTS";

/// One device as `lsblk` reports it, with whatever sits on top of it nested inside.
#[derive(Debug, Default, Clone)]
pub(crate) struct Raw {
    name: String,
    path: String,
    kind: String,
    size: u64,
    model: Option<String>,
    transport: Option<String>,
    rotational: bool,
    removable: bool,
    fstype: Option<String>,
    label: Option<String>,
    mountpoints: Vec<String>,
    children: Vec<Raw>,
}

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

fn number(v: &serde_json::Value, key: &str) -> u64 {
    v[key]
        .as_u64()
        .or_else(|| v[key].as_str().and_then(|s| s.parse().ok()))
        .unwrap_or(0)
}

fn raw(v: &serde_json::Value) -> Raw {
    let mut mountpoints: Vec<String> = v["mountpoints"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|m| m.as_str().map(str::to_owned))
        .collect();
    mountpoints.extend(text(v, "mountpoint"));
    mountpoints.retain(|m| !m.starts_with('['));
    let name = text(v, "name").unwrap_or_default();
    Raw {
        path: text(v, "path").unwrap_or_else(|| format!("/dev/{name}")),
        name,
        kind: text(v, "type").unwrap_or_default(),
        size: number(v, "size"),
        model: text(v, "model"),
        transport: text(v, "tran"),
        rotational: flag(v, "rota"),
        removable: flag(v, "rm") || flag(v, "hotplug"),
        fstype: text(v, "fstype"),
        label: text(v, "label"),
        mountpoints,
        children: v["children"]
            .as_array()
            .into_iter()
            .flatten()
            .map(raw)
            .collect(),
    }
}

pub(crate) fn parse_lsblk(json: &str) -> Vec<Raw> {
    serde_json::from_str::<serde_json::Value>(json)
        .map(|root| {
            root["blockdevices"]
                .as_array()
                .into_iter()
                .flatten()
                .map(raw)
                .collect()
        })
        .unwrap_or_default()
}

/// Filesystems that only hold other volumes: nothing to browse until those are opened.
const CONTAINERS: &[&str] = &[
    "LVM2_member",
    "crypto_LUKS",
    "linux_raid_member",
    "zfs_member",
    "bcache",
    "isw_raid_member",
    "ddf_raid_member",
];

fn role(raw: &Raw) -> &'static str {
    match raw.fstype.as_deref() {
        Some("swap") => "swap",
        Some(t) if CONTAINERS.contains(&t) => "container",
        Some(_) => "filesystem",
        None if !raw.children.is_empty() => "container",
        None => "empty",
    }
}

/// The shortest mount point stands for a filesystem mounted in several places (btrfs subvolumes).
fn mount_of(raw: &Raw) -> Option<String> {
    raw.mountpoints.iter().min_by_key(|m| m.len()).cloned()
}

fn volume(raw: &Raw, usage: &HashMap<String, Usage>) -> Volume {
    let mount = mount_of(raw);
    let role = role(raw);
    let space = mount
        .as_ref()
        .and_then(|m| usage.get(m))
        .copied()
        .unwrap_or_default();
    Volume {
        path: raw.path.clone(),
        name: raw.name.clone(),
        size: raw.size,
        fstype: raw.fstype.clone(),
        label: raw.label.clone(),
        mountable: role == "filesystem" && mount.is_none(),
        mount,
        used: space.used,
        available: space.available,
        role,
        children: raw.children.iter().map(|c| volume(c, usage)).collect(),
    }
}

fn disk_kind(raw: &Raw) -> &'static str {
    if raw.kind == "rom" {
        "optical"
    } else if raw.transport.as_deref() == Some("usb") {
        "usb"
    } else if raw.transport.as_deref() == Some("nvme") || raw.name.starts_with("nvme") {
        "nvme"
    } else if raw.rotational {
        "hdd"
    } else {
        "ssd"
    }
}

/// Real drives only: loop images (snaps), compressed RAM and empty card-reader slots aren't.
fn is_drive(raw: &Raw) -> bool {
    matches!(raw.kind.as_str(), "disk" | "rom")
        && raw.size > 0
        && !["loop", "zram", "ram", "nbd"]
            .iter()
            .any(|p| raw.name.starts_with(p))
}

pub(crate) fn disks(raws: &[Raw], usage: &HashMap<String, Usage>) -> Vec<Disk> {
    raws.iter()
        .filter(|r| is_drive(r))
        .map(|r| Disk {
            path: r.path.clone(),
            name: r.name.clone(),
            model: r.model.clone(),
            size: r.size,
            kind: disk_kind(r),
            removable: r.removable,
            volumes: r.children.iter().map(|c| volume(c, usage)).collect(),
        })
        .collect()
}

const NETWORK: &[&str] = &[
    "nfs",
    "nfs4",
    "cifs",
    "smb3",
    "smbfs",
    "ceph",
    "glusterfs",
    "afs",
    "9p",
    "davfs",
    "lustre",
    "beegfs",
    "ocfs2",
    "gfs2",
    "orangefs",
    "coda",
    "ncpfs",
    "fuse.sshfs",
    "fuse.rclone",
    "fuse.s3fs",
    "fuse.gcsfuse",
    "fuse.davfs2",
    "fuse.curlftpfs",
    "fuse.cephfs",
    "fuse.glusterfs",
];

/// A hung share must not freeze the page, so each is asked with a short deadline.
fn network_usage(target: &str) -> Option<Usage> {
    let text = run("timeout", &["2", "df", "-B1", DF_COLUMNS, "--", target])?;
    parse_df(&text).into_values().next()
}

fn shares() -> Result<Option<Disk>> {
    let text = std::fs::read_to_string("/proc/self/mountinfo")
        .map_err(|e| AppError::Other(format!("Couldn't read the mount table: {e}")))?;
    let mut volumes: Vec<Volume> = visible(parse_mountinfo(&text))
        .into_iter()
        .filter(|m| NETWORK.contains(&m.fstype.as_str()))
        .map(|m| {
            let space = network_usage(&m.target).unwrap_or_default();
            Volume {
                path: m.target.clone(),
                name: m.source.clone(),
                size: space.size.unwrap_or(0),
                fstype: Some(m.fstype),
                label: None,
                mount: Some(m.target),
                used: space.used,
                available: space.available,
                role: "filesystem",
                mountable: false,
                children: Vec::new(),
            }
        })
        .collect();
    volumes.sort_by(|a, b| a.path.cmp(&b.path));
    Ok((!volumes.is_empty()).then(|| Disk {
        path: "network".into(),
        name: "Network shares".into(),
        model: None,
        size: volumes.iter().map(|v| v.size).sum(),
        kind: "network",
        removable: false,
        volumes,
    }))
}

fn walk<'a>(volumes: &'a [Volume], out: &mut Vec<&'a Volume>) {
    for v in volumes {
        out.push(v);
        walk(&v.children, out);
    }
}

pub(crate) fn overview(disks: &[Disk]) -> Overview {
    let mut o = Overview::default();
    let mut seen = HashSet::new();
    for disk in disks.iter().filter(|d| d.kind != "network") {
        o.disks += 1;
        o.capacity += disk.size;
        let mut all = Vec::new();
        walk(&disk.volumes, &mut all);
        for v in all.into_iter().filter(|v| v.role == "filesystem") {
            if v.mount.is_some() {
                o.mounted += 1;
                if let (Some(used), Some(free), true) = (v.used, v.available, seen.insert(&v.path))
                {
                    o.used += used;
                    o.available += free;
                }
            } else {
                o.unmounted += 1;
            }
        }
    }
    o
}

pub(crate) fn lsblk() -> Vec<Raw> {
    run("lsblk", &["-J", "-b", "-o", LSBLK_COLUMNS])
        .map(|t| parse_lsblk(&t))
        .unwrap_or_default()
}

pub(crate) fn snapshot() -> Result<Devices> {
    let raws = lsblk();
    if raws.is_empty() {
        return Err(AppError::MissingTool("lsblk"));
    }
    let usage: HashMap<String, Usage> = run("df", &["-B1", "-a", "-l", DF_COLUMNS])
        .map(|t| parse_df(&t))
        .unwrap_or_default();
    let mut all = disks(&raws, &usage);
    all.extend(shares()?);
    Ok(Devices {
        overview: overview(&all),
        disks: all,
    })
}

/// Looks a device up by path anywhere in the tree.
pub(crate) fn lookup<'a>(raws: &'a [Raw], path: &str) -> Option<&'a Raw> {
    raws.iter().find_map(|r| {
        if r.path == path {
            Some(r)
        } else {
            lookup(&r.children, path)
        }
    })
}

/// Whether `path` is a filesystem that exists and isn't mounted: the only thing worth mounting here.
pub(crate) fn can_mount(raws: &[Raw], path: &str) -> std::result::Result<(), String> {
    let Some(raw) = lookup(raws, path) else {
        return Err(format!("{path} isn't a storage device on this computer"));
    };
    if role(raw) != "filesystem" {
        return Err(format!("{path} has no filesystem to mount"));
    }
    if mount_of(raw).is_some() {
        return Err(format!("{path} is already mounted"));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    const LSBLK: &str = r#"{"blockdevices":[
      {"name":"loop1","path":"/dev/loop1","type":"loop","size":77574144,"fstype":"squashfs","mountpoints":["/snap/core22/1"]},
      {"name":"zram0","path":"/dev/zram0","type":"disk","size":8589934592,"fstype":"swap","mountpoints":["[SWAP]"]},
      {"name":"sr0","path":"/dev/sr0","type":"rom","size":0,"model":"DVD","mountpoints":[null]},
      {"name":"sda","path":"/dev/sda","type":"disk","size":1000204886016,"model":"WDC WD10","tran":"sata","rota":true,"rm":false,"hotplug":false,"mountpoints":[null],"children":[
         {"name":"sda1","path":"/dev/sda1","type":"part","size":537919488,"fstype":"vfat","mountpoints":[null]},
         {"name":"sda2","path":"/dev/sda2","type":"part","size":999000000000,"fstype":"ntfs","label":"New Volume","mountpoints":[null]}]},
      {"name":"sdb","path":"/dev/sdb","type":"disk","size":2000398934016,"model":"ST2000","tran":"sata","rota":true,"mountpoints":[null],"children":[
         {"name":"sdb1","path":"/dev/sdb1","type":"part","size":134217728,"mountpoints":[null]}]},
      {"name":"nvme0n1","path":"/dev/nvme0n1","type":"disk","size":500107862016,"model":"Samsung 980","tran":"nvme","rota":false,"mountpoints":[null],"children":[
         {"name":"nvme0n1p1","path":"/dev/nvme0n1p1","type":"part","size":1073741824,"fstype":"vfat","mountpoints":["/boot/efi"]},
         {"name":"nvme0n1p2","path":"/dev/nvme0n1p2","type":"part","size":16106127360,"fstype":"swap","mountpoints":["[SWAP]"]},
         {"name":"nvme0n1p3","path":"/dev/nvme0n1p3","type":"part","size":482000000000,"fstype":"crypto_LUKS","mountpoints":[null],"children":[
            {"name":"cryptroot","path":"/dev/mapper/cryptroot","type":"crypt","size":481900000000,"fstype":"btrfs","mountpoints":["/","/home"]}]}]},
      {"name":"sdc","path":"/dev/sdc","type":"disk","size":31914983424,"model":"Flash","tran":"usb","rm":"1","hotplug":"1","mountpoints":[null],"children":[
         {"name":"sdc1","path":"/dev/sdc1","type":"part","size":31913000000,"fstype":"exfat","label":"STICK","mountpoints":["/media/me/STICK"]}]}]}"#;

    fn usage() -> HashMap<String, Usage> {
        let u = |size, used| Usage {
            size: Some(size),
            used: Some(used),
            available: Some(size - used),
        };
        HashMap::from([
            ("/".to_owned(), u(481_000_000_000, 100_000_000_000)),
            ("/boot/efi".to_owned(), u(1_000_000_000, 6_000_000)),
            (
                "/media/me/STICK".to_owned(),
                u(31_000_000_000, 11_000_000_000),
            ),
        ])
    }

    fn built() -> Vec<Disk> {
        disks(&parse_lsblk(LSBLK), &usage())
    }

    #[test]
    fn only_real_drives_are_listed_not_loops_zram_or_empty_slots() {
        let all = built();
        let names: Vec<&str> = all.iter().map(|d| d.name.as_str()).collect();
        assert_eq!(names, ["sda", "sdb", "nvme0n1", "sdc"]);
    }

    #[test]
    fn drives_are_told_apart() {
        let all = built();
        let kinds: Vec<&str> = all.iter().map(|d| d.kind).collect();
        assert_eq!(kinds, ["hdd", "hdd", "nvme", "usb"]);
        assert!(all[3].removable);
        assert!(!all[0].removable);
    }

    #[test]
    fn an_unmounted_filesystem_can_be_mounted_and_an_unformatted_partition_cannot() {
        let d = built();
        let ntfs = &d[0].volumes[1];
        assert_eq!(
            (ntfs.role, ntfs.mountable, ntfs.mount.as_deref()),
            ("filesystem", true, None)
        );
        assert_eq!(ntfs.label.as_deref(), Some("New Volume"));
        let msr = &d[1].volumes[0];
        assert_eq!((msr.role, msr.mountable), ("empty", false));
    }

    #[test]
    fn swap_is_not_browsable_and_a_container_holds_its_volumes() {
        let all = built();
        let nvme = &all[2];
        assert_eq!(nvme.volumes[1].role, "swap");
        assert_eq!(nvme.volumes[1].mount, None);
        let luks = &nvme.volumes[2];
        assert_eq!(luks.role, "container");
        assert_eq!(luks.children[0].role, "filesystem");
    }

    #[test]
    fn a_filesystem_mounted_twice_is_shown_at_its_shortest_path_with_its_space() {
        let all = built();
        let nvme = &all[2];
        let root = &nvme.volumes[2].children[0];
        assert_eq!(root.mount.as_deref(), Some("/"));
        assert_eq!(
            (root.used, root.available),
            (Some(100_000_000_000), Some(381_000_000_000))
        );
        assert!(!root.mountable);
    }

    #[test]
    fn the_overview_counts_drives_space_and_what_is_left_unmounted() {
        let o = overview(&built());
        assert_eq!(o.disks, 4);
        assert_eq!(
            o.capacity,
            1000204886016 + 2000398934016 + 500107862016 + 31914983424
        );
        assert_eq!((o.mounted, o.unmounted), (3, 2));
        assert_eq!(o.used, 100_000_000_000 + 6_000_000 + 11_000_000_000);
    }

    #[test]
    fn only_an_unmounted_filesystem_passes_the_mount_check() {
        let raws = parse_lsblk(LSBLK);
        assert!(can_mount(&raws, "/dev/sda2").is_ok());
        assert!(can_mount(&raws, "/dev/sda1").is_ok());
        assert!(can_mount(&raws, "/dev/sdb1")
            .unwrap_err()
            .contains("no filesystem"));
        assert!(can_mount(&raws, "/dev/nvme0n1p2")
            .unwrap_err()
            .contains("no filesystem"));
        assert!(can_mount(&raws, "/dev/sdc1")
            .unwrap_err()
            .contains("already mounted"));
        assert!(can_mount(&raws, "/dev/nope")
            .unwrap_err()
            .contains("isn't a storage device"));
        assert!(can_mount(&raws, "/etc/passwd").is_err());
        assert!(lookup(&raws, "/dev/sda2").is_some());
    }

    #[test]
    fn bad_json_gives_nothing() {
        assert!(parse_lsblk("not json").is_empty());
    }

    #[test]
    fn every_drive_on_this_machine_is_listed_once() {
        let snapshot = snapshot().unwrap();
        let mut seen = HashSet::new();
        let mut volumes = Vec::new();
        for d in &snapshot.disks {
            assert!(seen.insert(d.path.clone()), "{} is listed twice", d.path);
            walk(&d.volumes, &mut volumes);
        }
        for v in volumes {
            assert!(seen.insert(v.path.clone()), "{} is listed twice", v.path);
        }
        assert!(snapshot.overview.disks >= 1);
    }
}
