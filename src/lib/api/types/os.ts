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

export interface OsMemory {
  total: number;
  used: number;
  available: number;
  swap_total: number;
  swap_used: number;
  details: Detail[];
}

export interface OsSecurity {
  apparmor: string | null;
  selinux: string | null;
  lockdown: string | null;
  secure_boot: string | null;
  details: Detail[];
}

export interface CgroupRow {
  name: string;
  /** Processes inside it and its children */
  pids: number | null;
  /** Memory charged to it */
  memory: number | null;
  /** Groups nested beneath it */
  groups: number | null;
}

export interface Cgroups {
  version: string;
  controllers: string[];
  groups: number;
  /** The count hit its ceiling, so the real number is higher */
  capped: boolean;
  /** The groups directly under the root, biggest first */
  top: CgroupRow[];
}

export interface NetInterface {
  name: string;
  /** "ethernet", "wifi", "bridge", "virtual" or "loopback" */
  kind: string;
  state: string;
  mac: string | null;
  mtu: number | null;
  speed: string | null;
  ipv4: string[];
  ipv6: number;
  rx: number;
  tx: number;
}

export interface OsNetwork {
  /** Real interfaces that are up, and how many exist (the loopback doesn't count) */
  up: number;
  total: number;
  default_route: string | null;
  dns: string[];
  listening_tcp: number[];
  established: number;
  interfaces: NetInterface[];
  details: Detail[];
}

export interface ConnRow {
  proto: string;
  local: string;
  remote: string;
  state: string;
}

export interface Connections {
  established: number;
  listening: number;
  time_wait: number;
  rows: ConnRow[];
}

export interface OsLogs {
  available: boolean;
  size: string | null;
  boots: number | null;
  /** The error-level lines of this boot, oldest first */
  errors: string[];
  truncated: boolean;
  note: string | null;
}

export interface LoginSession {
  id: string;
  user: string;
  /** "wayland", "x11" or "tty" */
  kind: string;
  class: string;
  /** The TTY, seat or remote host it comes from */
  place: string;
  remote: boolean;
  since: string | null;
  state: string;
}

export interface OsLogins {
  available: boolean;
  sessions: LoginSession[];
}

export interface ContainerRow {
  name: string;
  image: string;
  state: string;
  status: string;
}

export interface Containers {
  /** "docker" or "podman"; null when neither is installed */
  runtime: string | null;
  note: string | null;
  running: number;
  containers: ContainerRow[];
  images: { name: string; size: string }[];
}

export interface VmRow {
  name: string;
  state: string;
  manager: string;
}

export interface Vms {
  managers: string[];
  running: number;
  vms: VmRow[];
  note: string | null;
}

export interface DockerDf {
  kind: string;
  count: string;
  size: string;
  reclaimable: string;
}

export interface VirtSlice {
  name: string;
  groups: number;
  pids: number | null;
  memory: number | null;
}

export interface VirtOverview {
  /** "Intel VT-x" or "AMD-V" */
  cpu: string | null;
  kvm_device: boolean;
  kvm_module: string | null;
  nested: boolean | null;
  inside_vm: string | null;
  inside_container: string | null;
  /** Overlay filesystems mounted right now: running containers' roots */
  overlay_mounts: number;
  /** Whether the kernel passes packets on, which guest NAT depends on */
  ip_forward: boolean;
  docker_df: DockerDf[];
  slices: VirtSlice[];
}
