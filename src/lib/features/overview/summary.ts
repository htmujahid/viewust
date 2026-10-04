import type { Component, Detail, HardwareInfo, Peripheral } from "$lib/api/types";

export interface Group {
  key: string;
  title: string;
  items: Component[];
  cost: number;
}

const GROUPS: { key: string; title: string; kinds: string[] }[] = [
  { key: "board", title: "Motherboard", kinds: ["board"] },
  { key: "cpu", title: "Processor", kinds: ["cpu"] },
  { key: "memory", title: "Memory", kinds: ["ram"] },
  { key: "cooling", title: "Cooling", kinds: ["fan"] },
  { key: "graphics", title: "Graphics", kinds: ["gpu"] },
  { key: "storage", title: "Storage", kinds: ["hdd", "ssd", "nvme", "storage"] },
  { key: "network", title: "Network adapters", kinds: ["nic", "wireless"] },
  { key: "sound", title: "Sound", kinds: ["soundcard", "audio"] },
  { key: "power", title: "Power supply", kinds: ["psu"] },
];

export function groupComponents(components: readonly Component[]): Group[] {
  const known = new Set(GROUPS.flatMap((g) => g.kinds));
  const groups = [
    ...GROUPS.map((g) => ({
      key: g.key,
      title: g.title,
      items: components.filter((c) => g.kinds.includes(c.kind)),
    })),
    { key: "other", title: "Other", items: components.filter((c) => !known.has(c.kind)) },
  ];
  return groups
    .filter((g) => g.items.length > 0)
    .map((g) => ({ ...g, cost: 3 + g.items.length * 2 }));
}

export function pick(rows: readonly Detail[], section: string, label: string): string | null {
  return rows.find((r) => r.section === section && r.label === label)?.value ?? null;
}

const KIND_NAMES: Record<string, [string, string]> = {
  keyboard: ["keyboard", "keyboards"],
  mouse: ["mouse", "mice"],
  webcam: ["webcam", "webcams"],
  printer: ["printer", "printers"],
  storage: ["storage drive", "storage drives"],
  wireless: ["wireless receiver", "wireless receivers"],
  headphones: ["headset", "headsets"],
  microphone: ["microphone", "microphones"],
  gamepad: ["game controller", "game controllers"],
  phone: ["phone", "phones"],
  hub: ["USB hub", "USB hubs"],
  securitykey: ["security key", "security keys"],
};

export function peripheralKinds(peripherals: readonly Peripheral[]): string[] {
  const counts = new Map<string, number>();
  for (const p of peripherals) counts.set(p.kind, (counts.get(p.kind) ?? 0) + 1);
  return [...counts]
    .sort((a, b) => b[1] - a[1] || a[0].localeCompare(b[0]))
    .map(([kind, n]) => {
      const names = KIND_NAMES[kind];
      const label = names ? names[n === 1 ? 0 : 1] : n === 1 ? kind : `${kind} devices`;
      return `${n} ${label}`;
    });
}

const CONNECTIVITY: Record<string, string> = {
  full: "Online",
  limited: "Limited connection",
  portal: "Sign-in needed",
  none: "Offline",
  unknown: "Unknown",
};

export function internetSummary(info: HardwareInfo): { text: string; ok: boolean } | null {
  const c = info.connection;
  if (!c) return null;
  return {
    text: `${CONNECTIVITY[c.connectivity] ?? "Unknown"} · ${c.link_label}`,
    ok: c.connectivity === "full",
  };
}

export function displaySummary(info: HardwareInfo): string[] {
  return info.displays.map(
    (d) => `${d.name} · ${d.width}×${d.height}${d.primary ? " (main)" : ""}`,
  );
}
