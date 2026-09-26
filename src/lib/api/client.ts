/**
 * The only place the frontend talks to the Rust backend.
 *
 * Everything else calls `api.*`. That keeps command names and argument names in
 * one file, gives every call a return type, and lets a missing backend (opening
 * the page in a plain browser) fail with a message people can act on.
 */
import { invoke } from "@tauri-apps/api/core";

import type {
  Detail,
  HardwareInfo,
  MemoryModules,
  ProcessDetail,
  Sample,
  Snapshot,
  SystemInfo,
} from "$lib/api/types";

/** True inside the desktop window, or once the mock backend is installed. */
export function backendAvailable(): boolean {
  return typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;
}

async function call<T>(command: string, args?: Record<string, unknown>): Promise<T> {
  if (!backendAvailable()) {
    throw new Error("The desktop backend isn't running. Start the app with `pnpm tauri dev`.");
  }
  return invoke<T>(command, args);
}

/** Turns anything thrown by a command into text for the screen. */
export function errorMessage(error: unknown): string {
  return error instanceof Error ? error.message : String(error);
}

export const api = {
  /** Everything plugged in from outside, plus the internet connection. */
  hardwareInfo: () => call<HardwareInfo>("hardware_info"),
  /** The deep technical report for one node of the map. */
  deviceReport: (id: string) => call<Detail[]>("device_report", { id }),

  /** The computer's internal parts. */
  systemInfo: () => call<SystemInfo>("system_info"),
  /** Asks the desktop for administrator permission, then reads each RAM module. */
  readMemoryModules: () => call<MemoryModules>("read_memory_modules"),

  processList: () => call<Snapshot>("process_list"),
  processDetail: (pid: number) => call<ProcessDetail>("process_detail", { pid }),

  /** One second's readings of CPU, memory, drives, graphics and network. */
  monitorSample: () => call<Sample>("monitor_sample"),
} as const;
