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
