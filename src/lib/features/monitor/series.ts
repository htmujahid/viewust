import type { Sample } from "$lib/api/types";

export const COLOR_1 = "var(--series-1)";
export const COLOR_2 = "var(--series-2)";
export const PALETTE = [1, 2, 3, 4, 5, 6].map((n) => `var(--series-${n})`);
export const colorAt = (i: number) => PALETTE[i % PALETTE.length];

export const percent = (v: number) => `${v.toFixed(v >= 10 || v === 0 ? 0 : 1)}%`;
export const last = (xs: (number | null)[]) => xs.at(-1) ?? 0;

type Values = (number | null)[];

export const cpuLoad = (s: Sample[]): Values => s.map((x) => x.cpu.total);
export const cpuClock = (s: Sample[]): Values => s.map((x) => x.cpu.freq_mhz);
export const cpuTemp = (s: Sample[]): Values => s.map((x) => x.cpu.temperature);

export const memUsed = (s: Sample[]): Values =>
  s.map((x) => (x.memory.used / x.memory.total) * 100);
export const memCached = (s: Sample[]): Values =>
  s.map((x) => (x.memory.cached / x.memory.total) * 100);
export const swapUsed = (s: Sample[]): Values =>
  s.map((x) => (x.memory.swap_total ? (x.memory.swap_used / x.memory.swap_total) * 100 : 0));

export const diskRead = (s: Sample[], name?: string): Values =>
  s.map((x) => x.disks.filter((d) => !name || d.name === name).reduce((n, d) => n + d.read_bps, 0));
export const diskWrite = (s: Sample[], name?: string): Values =>
  s.map((x) =>
    x.disks.filter((d) => !name || d.name === name).reduce((n, d) => n + d.write_bps, 0),
  );
export const diskBusy = (s: Sample[], name: string): Values =>
  s.map((x) => x.disks.find((d) => d.name === name)?.busy ?? null);

const gpuOf = (x: Sample, id: string) => x.gpus.find((g) => g.id === id);
export const gpuUtil = (s: Sample[], id: string): Values =>
  s.map((x) => gpuOf(x, id)?.util ?? null);
export const gpuMem = (s: Sample[], id: string): Values =>
  s.map((x) => {
    const g = gpuOf(x, id);
    return g?.memory_used != null && g.memory_total ? (g.memory_used / g.memory_total) * 100 : null;
  });
export const gpuTemp = (s: Sample[], id: string): Values =>
  s.map((x) => gpuOf(x, id)?.temperature ?? null);
export const gpuPower = (s: Sample[], id: string): Values =>
  s.map((x) => gpuOf(x, id)?.power ?? null);
export const gpuClock = (s: Sample[], id: string): Values =>
  s.map((x) => gpuOf(x, id)?.core_mhz ?? null);
export const gpuFan = (s: Sample[], id: string): Values => s.map((x) => gpuOf(x, id)?.fan ?? null);

export const cpuWatts = (s: Sample[]): Values => s.map((x) => x.power.cpu_watts);
export const batteryPercent = (s: Sample[], name: string): Values =>
  s.map((x) => x.power.batteries.find((b) => b.name === name)?.percent ?? null);
export const batteryWatts = (s: Sample[], name: string): Values =>
  s.map((x) => x.power.batteries.find((b) => b.name === name)?.watts ?? null);
export const fanRpm = (s: Sample[], id: string): Values =>
  s.map((x) => x.thermal.fans.find((f) => f.id === id)?.rpm ?? null);

export const netDown = (s: Sample[], name?: string): Values =>
  s.map((x) => x.net.filter((n) => !name || n.name === name).reduce((a, n) => a + n.rx_bps, 0));
export const netUp = (s: Sample[], name?: string): Values =>
  s.map((x) => x.net.filter((n) => !name || n.name === name).reduce((a, n) => a + n.tx_bps, 0));

export const mhz = (v: number) =>
  v >= 1000 ? `${(v / 1000).toFixed(1)} GHz` : `${v.toFixed(0)} MHz`;
export const celsius = (v: number) => `${v.toFixed(0)} °C`;
export const watts = (v: number) => `${v.toFixed(v >= 10 ? 0 : 1)} W`;
