import { invoke } from "@tauri-apps/api/core";

import type {
  AudioSample,
  Cgroups,
  Connections,
  Containers,
  Detail,
  EnvVar,
  HealthReport,
  DirectoryUsage,
  DiskDevices,
  Filesystems,
  HardwareInfo,
  KernelModule,
  MemoryModules,
  ModuleInfo,
  OsLogins,
  OsLogs,
  OsMemory,
  OsNetwork,
  OsSecurity,
  OsSummary,
  Packages,
  ProcessDetail,
  Accounts,
  Namespaces,
  ServiceAction,
  ServiceDetail,
  ServiceSnapshot,
  Sample,
  Signal,
  SpeedTest,
  VirtOverview,
  Vms,
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

  diskDevices: () => call<DiskDevices>("disk_devices"),
  directoryUsage: (path: string, refresh = false) =>
    call<DirectoryUsage>("directory_usage", { path, refresh }),
  pathOpen: (path: string) => call<void>("path_open", { path }),
  /** Mounts a filesystem and returns where it is now. */
  diskMount: (device: string) => call<string>("disk_mount", { device }),
  filesystemList: () => call<Filesystems>("filesystem_list"),

  serviceList: () => call<ServiceSnapshot>("service_list"),
  serviceDetail: (unit: string) => call<ServiceDetail>("service_detail", { unit }),
  serviceAction: (unit: string, action: ServiceAction) =>
    call<void>("service_action", { unit, action }),

  osSummary: () => call<OsSummary>("os_summary"),
  kernelModules: () => call<KernelModule[]>("kernel_modules"),
  kernelModuleInfo: (name: string) => call<ModuleInfo>("kernel_module_info", { name }),
  osPackages: () => call<Packages>("os_packages"),
  osEnvironment: () => call<EnvVar[]>("os_environment"),
  osMemory: () => call<OsMemory>("os_memory"),
  osSecurity: () => call<OsSecurity>("os_security"),
  osCgroups: () => call<Cgroups>("os_cgroups"),
  osNetwork: () => call<OsNetwork>("os_network"),
  osConnections: () => call<Connections>("os_connections"),
  osLogs: () => call<OsLogs>("os_logs"),
  osLogins: () => call<OsLogins>("os_logins"),

  monitorSample: () => call<Sample>("monitor_sample"),
  audioSample: () => call<AudioSample>("audio_sample"),
  /** Measures the line by really using it; takes about half a minute. */
  speedTest: () => call<SpeedTest>("speed_test"),

  healthReport: () => call<HealthReport>("health_report"),
  osContainers: () => call<Containers>("os_containers"),
  osVms: () => call<Vms>("os_vms"),
  virtOverview: () => call<VirtOverview>("virt_overview"),
} as const;
