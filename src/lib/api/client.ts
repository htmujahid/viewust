import { invoke } from "@tauri-apps/api/core";

import type {
  Detail,
  FilesystemAction,
  FilesystemSnapshot,
  HardwareInfo,
  MemoryModules,
  ProcessDetail,
  Accounts,
  Namespaces,
  ServiceAction,
  ServiceDetail,
  ServiceSnapshot,
  Sample,
  Signal,
  Snapshot,
  SystemInfo,
} from "$lib/api/types";

export function backendAvailable(): boolean {
  return typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;
}

async function call<T>(command: string, args?: Record<string, unknown>): Promise<T> {
  if (!backendAvailable()) {
    throw new Error("The desktop backend isn't running. Start the app with `pnpm tauri dev`.");
  }
  return invoke<T>(command, args);
}

export function errorMessage(error: unknown): string {
  return error instanceof Error ? error.message : String(error);
}

export const api = {
  hardwareInfo: () => call<HardwareInfo>("hardware_info"),
  deviceReport: (id: string) => call<Detail[]>("device_report", { id }),

  systemInfo: () => call<SystemInfo>("system_info"),
  readMemoryModules: () => call<MemoryModules>("read_memory_modules"),

  processList: () => call<Snapshot>("process_list"),
  processDetail: (pid: number) => call<ProcessDetail>("process_detail", { pid }),
  processSignal: (pid: number, signal: Signal) => call<void>("process_signal", { pid, signal }),

  accountList: () => call<Accounts>("account_list"),
  namespaceList: () => call<Namespaces>("namespace_list"),

  filesystemList: () => call<FilesystemSnapshot>("filesystem_list"),
  filesystemAction: (mount: string, action: FilesystemAction) =>
    call<void>("filesystem_action", { mount, action }),

  serviceList: () => call<ServiceSnapshot>("service_list"),
  serviceDetail: (unit: string) => call<ServiceDetail>("service_detail", { unit }),
  serviceAction: (unit: string, action: ServiceAction) =>
    call<void>("service_action", { unit, action }),

  monitorSample: () => call<Sample>("monitor_sample"),
} as const;
