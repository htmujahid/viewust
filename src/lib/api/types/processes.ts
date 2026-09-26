import type { Detail } from "./common";

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
