export type DiskKind = "nvme" | "ssd" | "hdd" | "usb" | "optical" | "network";

/**
 * filesystem: has files; swap and empty: nothing to browse;
 * container: only holds other volumes (an encrypted or LVM layer)
 */
export type VolumeRole = "filesystem" | "swap" | "container" | "empty";

/** A partition, logical volume or encrypted layer on a disk, or a network share. */
export interface DiskVolume {
  /** `/dev/sda2`, or the mount point for a share. Unique across the list. */
  path: string;
  name: string;
  size: number;
  fstype: string | null;
  label: string | null;
  mount: string | null;
  used: number | null;
  available: number | null;
  role: VolumeRole;
  /** A filesystem that isn't mounted, so it can be mounted to browse it */
  mountable: boolean;
  children: DiskVolume[];
}

/** A physical drive, or the group of network shares. */
export interface Disk {
  path: string;
  name: string;
  model: string | null;
  size: number;
  kind: DiskKind;
  removable: boolean;
  volumes: DiskVolume[];
}

export interface DiskOverview {
  /** Combined size of the physical drives */
  capacity: number;
  used: number;
  available: number;
  disks: number;
  mounted: number;
  unmounted: number;
}

export interface DiskDevices {
  overview: DiskOverview;
  disks: Disk[];
}

export type EntryKind = "dir" | "file" | "link" | "other";

export interface UsageEntry {
  name: string;
  path: string;
  kind: EntryKind;
  /** Space taken on disk, in bytes */
  size: number;
  /** Files inside, for a folder */
  files: number;
  unreadable: boolean;
  /** Another filesystem mounted here; its space isn't counted in the parent */
  mount: boolean;
}

export interface DirectoryUsage {
  path: string;
  total: number;
  /** Biggest first */
  entries: UsageEntry[];
  hidden_count: number;
  hidden_size: number;
  unreadable: boolean;
  /** The scan hit its time limit, so the figures are lower than the truth */
  incomplete: boolean;
  took_ms: number;
}

export interface FsRow {
  mount: string;
  source: string;
  fstype: string;
  size: number | null;
  used: number | null;
  available: number | null;
  /** No space of its own: a kernel view or memory-backed mount */
  pseudo: boolean;
}

export interface Filesystems {
  mounted: number;
  real: number;
  rows: FsRow[];
}
