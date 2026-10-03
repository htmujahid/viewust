import type { Detail } from "./common";

export interface ServiceRow {
  unit: string;
  name: string;
  description: string;
  load: string;
  active: string;
  sub: string;
  enabled: string;
  main_pid: number | null;
  memory: number | null;
}

export interface ServiceOverview {
  total: number;
  running: number;
  exited: number;
  failed: number;
  inactive: number;
  enabled: number;
  memory: number;
}

export interface ServiceSnapshot {
  available: boolean;
  overview: ServiceOverview;
  services: ServiceRow[];
}

export interface ServiceDetail {
  unit: string;
  found: boolean;
  details: Detail[];
  logs: string[];
  logs_note: string | null;
}
