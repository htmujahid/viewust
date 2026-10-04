/// What sort of storage a mounted filesystem sits on. The page groups by this.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum Kind {
    Disk,
    Removable,
    Optical,
    Network,
    Memory,
    Image,
    Overlay,
    Virtual,
}

impl Kind {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Kind::Disk => "disk",
            Kind::Removable => "removable",
            Kind::Optical => "optical",
            Kind::Network => "network",
            Kind::Memory => "memory",
            Kind::Image => "image",
            Kind::Overlay => "overlay",
            Kind::Virtual => "virtual",
        }
    }

    pub(crate) fn label(self) -> &'static str {
        match self {
            Kind::Disk => "Internal disk",
            Kind::Removable => "Removable drive",
            Kind::Optical => "Optical disc",
            Kind::Network => "Network share",
            Kind::Memory => "Memory (RAM)",
            Kind::Image => "Packed image",
            Kind::Overlay => "Layered (containers)",
            Kind::Virtual => "System / virtual",
        }
    }

    /// Lower sorts first: real storage before the plumbing.
    pub(crate) fn rank(self) -> u8 {
        self as u8
    }

    /// Whether its space counts towards "how much storage do I have".
    pub(crate) fn is_storage(self) -> bool {
        matches!(self, Kind::Disk | Kind::Removable | Kind::Network)
    }
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

const DISK: &[&str] = &[
    "ext2",
    "ext3",
    "ext4",
    "xfs",
    "btrfs",
    "zfs",
    "f2fs",
    "jfs",
    "reiserfs",
    "reiser4",
    "bcachefs",
    "nilfs2",
    "ntfs",
    "ntfs3",
    "fuseblk",
    "vfat",
    "exfat",
    "msdos",
    "hfs",
    "hfsplus",
    "apfs",
    "ufs",
    "minix",
    "bfs",
    "ocfs",
    "refs",
    "lvm2_member",
];

pub(crate) fn classify(fstype: &str, source: &str, removable: bool) -> Kind {
    if NETWORK.contains(&fstype) {
        return Kind::Network;
    }
    match fstype {
        "tmpfs" | "ramfs" | "devtmpfs" | "hugetlbfs" => return Kind::Memory,
        "iso9660" | "udf" => return Kind::Optical,
        "squashfs" | "erofs" | "cramfs" => return Kind::Image,
        "overlay" | "aufs" | "unionfs" => return Kind::Overlay,
        _ => {}
    }
    // Pool-based filesystems (zfs "tank/home") have no /dev path, so the type decides.
    if source.starts_with("/dev/") || DISK.contains(&fstype) {
        return if removable {
            Kind::Removable
        } else {
            Kind::Disk
        };
    }
    Kind::Virtual
}

/// A plain-words line about a filesystem type, for the details panel.
pub(crate) fn describe(fstype: &str) -> Option<&'static str> {
    Some(match fstype {
        "ext4" => "The default Linux filesystem: journaled, mature and fast",
        "ext3" => "Older journaled Linux filesystem",
        "ext2" => "Old Linux filesystem without a journal",
        "xfs" => "High-performance journaled filesystem, common on servers",
        "btrfs" => "Copy-on-write filesystem with snapshots, checksums and subvolumes",
        "zfs" => "Pooled storage with checksums, snapshots and built-in RAID",
        "f2fs" => "Flash-friendly filesystem for SSDs and phones",
        "bcachefs" => "Modern copy-on-write filesystem with caching and checksums",
        "ntfs" | "ntfs3" | "fuseblk" => "Windows filesystem",
        "vfat" | "msdos" => "FAT: the simple format used by USB sticks and the boot partition",
        "exfat" => "exFAT: the format for large USB sticks and SD cards shared with other systems",
        "iso9660" | "udf" => "Optical disc or disc image",
        "nfs" | "nfs4" => "Network File System share",
        "cifs" | "smb3" | "smbfs" => "Windows / Samba network share",
        "fuse.sshfs" => "Folder reached over SSH",
        "9p" | "virtiofs" | "vboxsf" | "vmhgfs" => "Folder shared by a virtual-machine host",
        "tmpfs" => "Lives in memory and is emptied when the computer restarts",
        "ramfs" => "Simple memory-backed filesystem that can't be limited in size",
        "devtmpfs" => "The /dev device files, created by the kernel",
        "squashfs" => "Compressed read-only image, used by snaps and live systems",
        "erofs" => "Compressed read-only image used by modern systems and Android",
        "overlay" => "Layers stacked on top of each other, used by containers",
        "proc" => "Kernel view of processes and settings, not stored on any drive",
        "sysfs" => "Kernel view of devices and drivers, not stored on any drive",
        "cgroup" | "cgroup2" => "Resource-control groups, not stored on any drive",
        "devpts" => "Terminal devices",
        "autofs" => "Placeholder that mounts a folder when something opens it",
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn real_disks_split_into_internal_and_removable() {
        assert_eq!(classify("ext4", "/dev/nvme0n1p2", false), Kind::Disk);
        assert_eq!(classify("vfat", "/dev/sdb1", true), Kind::Removable);
        assert_eq!(classify("ntfs3", "/dev/mapper/data", false), Kind::Disk);
        assert_eq!(classify("zfs", "tank/home", false), Kind::Disk);
    }

    #[test]
    fn network_shares_are_recognised_including_fuse_ones() {
        assert_eq!(classify("nfs4", "srv:/export", false), Kind::Network);
        assert_eq!(classify("cifs", "//nas/media", false), Kind::Network);
        assert_eq!(classify("fuse.sshfs", "me@box:/", false), Kind::Network);
    }

    #[test]
    fn memory_images_layers_and_discs_have_their_own_groups() {
        assert_eq!(classify("tmpfs", "tmpfs", false), Kind::Memory);
        assert_eq!(classify("squashfs", "/dev/loop3", false), Kind::Image);
        assert_eq!(classify("overlay", "overlay", false), Kind::Overlay);
        assert_eq!(classify("iso9660", "/dev/sr0", true), Kind::Optical);
    }

    #[test]
    fn kernel_plumbing_is_virtual() {
        for t in ["proc", "sysfs", "cgroup2", "devpts", "fuse.portal", "bpf"] {
            assert_eq!(classify(t, t, false), Kind::Virtual, "{t}");
        }
    }

    #[test]
    fn only_disks_and_shares_count_as_storage() {
        assert!(Kind::Disk.is_storage() && Kind::Network.is_storage());
        assert!(!Kind::Memory.is_storage() && !Kind::Virtual.is_storage());
    }

    #[test]
    fn ranks_put_real_storage_first() {
        assert!(Kind::Disk.rank() < Kind::Memory.rank());
        assert!(Kind::Memory.rank() < Kind::Virtual.rank());
    }
}
