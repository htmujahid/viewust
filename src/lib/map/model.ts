import type { Edge } from "@xyflow/svelte";

import type { Detail } from "$lib/api/types";
import type { Kind } from "$lib/illustrations/kinds";

export interface NodeInfo {
  id: string;
  kind: Kind;
  label: string;
  title: string;
  subtitle?: string;
  connection?: "Wired" | "Wireless";
  via?: string;
  details: Detail[];
}

export interface DeviceData extends Record<string, unknown> {
  kind: Kind;
  tooltip: string;
  wireless: boolean;
  cm?: number;
  caption?: { text: string; tone: "ok" | "warn" | "danger" };
  signal?: number;
  info: NodeInfo;
}

export interface ComputerData extends Record<string, unknown> {
  name: string;
  info: NodeInfo;
}

export type Side = "t" | "b" | "l" | "r";

export const KIND_LABELS: Record<Kind, string> = {
  computer: "Computer",
  monitor: "Display",
  keyboard: "Keyboard",
  mouse: "Mouse",
  webcam: "Webcam",
  audio: "Audio",
  microphone: "Microphone",
  gamepad: "Game controller",
  storage: "USB drive",
  printer: "Printer",
  hub: "USB hub",
  wireless: "Wireless receiver",
  phone: "Phone",
  securitykey: "Security key",
  generic: "USB device",
  board: "Motherboard",
  cpu: "Processor",
  ram: "Memory",
  gpu: "Graphics card",
  nvme: "NVMe drive",
  ssd: "Solid-state drive",
  hdd: "Hard disk",
  fan: "Fan",
  psu: "Power supply",
  nic: "Network adapter",
  soundcard: "Sound card",
  router: "Router",
  internet: "Internet",
};

export function infoRows(info: NodeInfo): Detail[] {
  return info.via
    ? [{ section: "Connection", label: "Connected through", value: info.via }, ...info.details]
    : info.details;
}

/** Where to watch a device of this kind live, when the monitor has a page for it. */
export function monitorUrl(kind: Kind): string | null {
  switch (kind) {
    case "computer":
      return "/monitor";
    case "cpu":
      return "/monitor/cpu";
    case "ram":
      return "/monitor/memory";
    case "gpu":
    case "monitor":
      return "/monitor/gpu";
    case "nvme":
    case "ssd":
    case "hdd":
    case "storage":
      return "/monitor/storage";
    case "nic":
    case "router":
    case "internet":
    case "wireless":
      return "/monitor/network";
    case "fan":
      return "/monitor/thermal";
    case "psu":
      return "/monitor/power";
    case "audio":
    case "soundcard":
    case "microphone":
      return "/monitor/audio";
    default:
      return null;
  }
}

export const deviceUrl = (id: string) =>
  id === "computer" ? "/system" : `/device/${encodeURIComponent(id)}`;

export const tooltip = (i: NodeInfo) => [i.label, i.title, i.subtitle].filter(Boolean).join(" · ");

export function edge(
  source: string,
  from: Side | string,
  target: string,
  to: Side,
  wireless: boolean,
): Edge {
  return {
    id: `${source}:${from}->${target}:${to}`,
    source,
    sourceHandle: `out-${from}`,
    target,
    targetHandle: `in-${to}`,
    type: wireless ? "default" : "smoothstep",
    animated: wireless,
    class: wireless ? "wireless" : "wired",
  };
}
