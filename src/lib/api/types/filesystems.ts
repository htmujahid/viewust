import type { Detail } from "./common";

export type FilesystemKind =
  "disk" | "removable" | "optical" | "network" | "memory" | "image" | "overlay" | "virtual";

export interface FilesystemRow {
  mount: string;
  source: string;
  fstype: string;
  kind: FilesystemKind;
  read_only: boolean;
  size: number | null;
  used: number | null;
  available: number | null;
  inodes_total: number | null;
  inodes_used: number | null;
  label: string | null;
  details: Detail[];
}

export interface FilesystemOverview {
  size: number;
  used: number;
  available: number;
  volumes: number;
}

export interface FilesystemSnapshot {
  overview: FilesystemOverview;
  filesystems: FilesystemRow[];
}

export type FilesystemAction = "open" | "unmount";
