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
  id: string;
  vendor: "nvidia" | "amd" | "intel";
  kind: "discrete" | "integrated" | "unknown";
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

export interface TempSensor {
  id: string;
  group: string;
  label: string;
  celsius: number;
  high: number | null;
  critical: number | null;
}

export interface FanSensor {
  id: string;
  group: string;
  label: string;
  rpm: number;
}

export interface Thermal {
  sensors: TempSensor[];
  fans: FanSensor[];
}

export interface Battery {
  name: string;
  percent: number;
  status: string;
  watts: number | null;
  seconds_left: number | null;
  health: number | null;
  cycles: number | null;
}

export interface Power {
  cpu_watts: number | null;
  cpu_readable: boolean;
  cpu_limit_sustained: number | null;
  cpu_limit_boost: number | null;
  ac_online: boolean | null;
  batteries: Battery[];
}

export interface Sample {
  t: number;
  cpu: CpuSample;
  memory: MemorySample;
  disks: DiskRate[];
  volumes: Volume[];
  gpus: GpuSample[];
  net: NetRate[];
  thermal: Thermal;
  power: Power;
}
