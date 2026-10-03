import { describe, expect, it } from "vitest";

import { fixtures } from "$lib/api/mock";

import { groups, groupSeries, hasPower, hasThermal, hottestSeries, readings } from "./thermal";

const samples = [1, 2, 3].map((tick) => fixtures.sample(tick));
const sample = samples[0];

describe("thermal", () => {
  it("adds an NVIDIA card's temperature, which has no hwmon sensor of its own", () => {
    const gpu = readings(sample).filter((r) => r.id.startsWith("gpu:"));
    expect(gpu).toHaveLength(1);
    expect(gpu[0].group).toBe("NVIDIA GeForce GTX 1060 3GB");
  });

  it("groups sensors by component and picks each group's hottest", () => {
    const cpu = groups(sample).find((g) => g.name === "CPU")!;
    expect(cpu.readings.length).toBeGreaterThan(1);
    expect(cpu.hottest.celsius).toBe(Math.max(...cpu.readings.map((r) => r.celsius)));
  });

  it("charts a group as its hottest sensor over time", () => {
    const series = groupSeries(samples, "CPU");
    expect(series).toHaveLength(samples.length);
    expect(series.every((v) => v !== null)).toBe(true);
    expect(groupSeries(samples, "nothing")).toEqual([null, null, null]);
  });

  it("follows the hottest reading overall", () => {
    const top = hottestSeries(samples)[0] as number;
    expect(top).toBe(Math.max(...readings(sample).map((r) => r.celsius)));
  });

  it("knows when there is nothing to show", () => {
    expect(hasThermal(null)).toBe(false);
    expect(hasThermal(sample)).toBe(true);
    const bare = {
      ...sample,
      gpus: [],
      thermal: { sensors: [], fans: [] },
      power: { ...sample.power, cpu_limit_sustained: null, cpu_limit_boost: null, batteries: [] },
    };
    expect(hasThermal(bare)).toBe(false);
    expect(hasPower(bare)).toBe(false);
    expect(hasPower(sample)).toBe(true);
  });
});
