import type { Sample } from "$lib/api/types";

export interface Reading {
  id: string;
  group: string;
  label: string;
  celsius: number;
  high: number | null;
  critical: number | null;
}

export interface Group {
  name: string;
  readings: Reading[];
  hottest: Reading;
}

export function readings(sample: Sample): Reading[] {
  const gpus: Reading[] = sample.gpus
    .filter((g) => g.vendor === "nvidia" && g.temperature !== null)
    .map((g) => ({
      id: `gpu:${g.id}`,
      group: g.name,
      label: "GPU core",
      celsius: g.temperature as number,
      high: null,
      critical: null,
    }));
  return [...sample.thermal.sensors, ...gpus];
}

export function groups(sample: Sample): Group[] {
  const byName = new Map<string, Reading[]>();
  for (const r of readings(sample)) byName.set(r.group, [...(byName.get(r.group) ?? []), r]);
  return [...byName].map(([name, list]) => ({
    name,
    readings: list,
    hottest: list.reduce((a, b) => (b.celsius > a.celsius ? b : a)),
  }));
}

export const groupSeries = (samples: Sample[], name: string): (number | null)[] =>
  samples.map((s) => {
    const list = readings(s).filter((r) => r.group === name);
    return list.length ? Math.max(...list.map((r) => r.celsius)) : null;
  });

export const hottestSeries = (samples: Sample[]): (number | null)[] =>
  samples.map((s) => {
    const all = readings(s);
    return all.length ? Math.max(...all.map((r) => r.celsius)) : null;
  });

export const hasThermal = (sample: Sample | null): boolean =>
  !!sample && readings(sample).length > 0;

export const hasPower = (sample: Sample | null): boolean =>
  !!sample &&
  (sample.power.cpu_readable ||
    sample.power.cpu_limit_sustained !== null ||
    sample.power.batteries.length > 0 ||
    sample.gpus.some((g) => g.power !== null));
