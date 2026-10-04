import type { Detail } from "./common";

export interface OsSummary {
  name: string;
  version: string | null;
  kernel: string;
  hostname: string;
  architecture: string;
  /** Seconds since the epoch */
  boot_time: number;
  details: Detail[];
}

export interface KernelModule {
  name: string;
  size: number;
  used_by: string[];
}

export interface ModuleInfo {
  name: string;
  found: boolean;
  details: Detail[];
}

export interface EnvVar {
  key: string;
  /** Empty when `hidden` */
  value: string;
  /** Looks like a secret, so its value is never sent to the window */
  hidden: boolean;
}

export interface PackageRow {
  name: string;
  version: string;
  arch: string | null;
}

export interface Packages {
  /** "apt (dpkg)", "rpm" or "pacman"; null when no known package manager is installed */
  manager: string | null;
  packages: PackageRow[];
}
