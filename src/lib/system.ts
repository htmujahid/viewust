export interface Detail {
  section: string;
  label: string;
  value: string;
}

export interface Peripheral {
  id: string;
  via: string | null;
  wireless: boolean;
  name: string;
  manufacturer: string | null;
  kind: string;
  connection: string;
  vendor_id: string;
  product_id: string;
  serial_number: string | null;
  details: Detail[];
}

export interface Display {
  name: string;
  connector: string | null;
  width_cm: number | null;
  width: number;
  height: number;
  scale_factor: number;
  primary: boolean;
  details: Detail[];
}

export interface HardwareInfo {
  computer_name: string;
  computer_details: Detail[];
  connection: Connection | null;
  peripherals: Peripheral[];
  displays: Display[];
}

export interface Component {
  id: string;
  kind: string;
  name: string;
  subtitle: string | null;
  details: Detail[];
}

export interface SystemInfo {
  computer_name: string;
  components: Component[];
}

export interface MemoryModules {
  modules: Component[];
  slots: number;
}

export interface ProcessRow {
  pid: number;
  parent: number | null;
  name: string;
  user: string;
  /** Percent of one core; can exceed 100. */
  cpu: number;
  /** In RAM now. */
  memory: number;
  /** Address space asked for. */
  virtual_memory: number;
  threads: number;
  state: string;
  kernel: boolean;
  run_time: number;
}

export interface Overview {
  processes: number;
  running: number;
  threads: number;
  cpu: number;
  cpu_count: number;
  load: [number, number, number];
  memory_total: number;
  memory_used: number;
  memory_available: number;
  swap_total: number;
  swap_used: number;
  uptime: number;
}

export interface Snapshot {
  overview: Overview;
  processes: ProcessRow[];
}

export interface MemoryBreakdown {
  requested: number;
  peak_requested: number;
  resident: number;
  peak_resident: number;
  anonymous: number;
  file_backed: number;
  shared: number;
  swapped: number;
  data: number;
  stack: number;
  code: number;
  libraries: number;
  page_tables: number;
  locked: number;
  proportional: number | null;
  private: number | null;
  shared_pages: number | null;
}

export interface ProcessDetail {
  pid: number;
  running: boolean;
  restricted: boolean;
  memory: MemoryBreakdown | null;
  regions: { name: string; size: number; resident: number }[];
  children: { pid: number; name: string; memory: number }[];
  details: Detail[];
}

export function formatDuration(total: number): string {
  const d = Math.floor(total / 86400);
  const h = Math.floor((total % 86400) / 3600);
  const m = Math.floor((total % 3600) / 60);
  if (d) return `${d}d ${h}h`;
  if (h) return `${h}h ${m}m`;
  return `${m}m ${Math.floor(total % 60)}s`;
}

export function formatBytes(bytes: number): string {
  if (bytes <= 0) return "0 B";
  const units = ["B", "KiB", "MiB", "GiB", "TiB"];
  const i = Math.min(Math.floor(Math.log(bytes) / Math.log(1024)), units.length - 1);
  return `${(bytes / 1024 ** i).toFixed(i === 0 ? 0 : 1)} ${units[i]}`;
}

export interface Connection {
  kind: "ethernet" | "wifi" | "other";
  interface: string;
  link_label: string;
  /** 0–100 for Wi-Fi, otherwise null. */
  signal: number | null;
  router_name: string;
  router_details: Detail[];
  connectivity: "full" | "limited" | "portal" | "none" | "unknown";
  internet_details: Detail[];
}

// ---- live monitor samples (one per second) ----------------------------------

export interface CpuSample {
  total: number;
  cores: number[];
  freq_mhz: number;
  temperature: number | null;
  load: [number, number, number];
}

export interface MemorySample {
  total: number;
  used: number;
  available: number;
  cached: number;
  swap_total: number;
  swap_used: number;
}

export interface DiskRate {
  name: string;
  model: string | null;
  read_bps: number;
  write_bps: number;
  busy: number;
}

export interface Volume {
  mount: string;
  file_system: string;
  total: number;
  used: number;
}

export interface GpuSample {
  name: string;
  util: number | null;
  memory_used: number | null;
  memory_total: number | null;
  temperature: number | null;
  power: number | null;
  power_limit: number | null;
  core_mhz: number | null;
  memory_mhz: number | null;
  fan: number | null;
}

export interface NetRate {
  name: string;
  kind: "wifi" | "ethernet";
  rx_bps: number;
  tx_bps: number;
  rx_total: number;
  tx_total: number;
  speed_mbps: number | null;
  default_route: boolean;
}

export interface Sample {
  t: number;
  cpu: CpuSample;
  memory: MemorySample;
  disks: DiskRate[];
  volumes: Volume[];
  gpus: GpuSample[];
  net: NetRate[];
}

/** 1536000 → "1.5 MB/s" (decimal, the way network and disk speeds are quoted). */
export function formatRate(bytesPerSecond: number): string {
  if (bytesPerSecond < 1) return "0 B/s";
  const units = ["B/s", "kB/s", "MB/s", "GB/s"];
  const i = Math.min(Math.floor(Math.log(bytesPerSecond) / Math.log(1000)), units.length - 1);
  const v = bytesPerSecond / 1000 ** i;
  return `${v >= 100 || i === 0 ? v.toFixed(0) : v.toFixed(1)} ${units[i]}`;
}
