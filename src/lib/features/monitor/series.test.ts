import { describe, expect, it } from "vitest";

import { fixtures } from "$lib/api/mock";

import * as m from "./series";

const samples = [1, 2, 3].map((tick) => fixtures.sample(tick));

describe("series", () => {
  it("yields one value per sample", () => {
    for (const series of [
      m.cpuLoad(samples),
      m.memUsed(samples),
      m.diskRead(samples),
      m.netDown(samples),
    ]) {
      expect(series).toHaveLength(samples.length);
    }
  });

  it("computes memory use as a percentage of the total", () => {
    for (const v of m.memUsed(samples)) {
      expect(v).toBeGreaterThan(0);
      expect(v).toBeLessThanOrEqual(100);
    }
  });

  it("sums drives, or picks one by name", () => {
    const all = m.diskRead(samples)[0] as number;
    const one = m.diskRead(samples, "sdc")[0] as number;
    expect(all).toBeGreaterThanOrEqual(one);
    expect(m.diskRead(samples, "nope")[0]).toBe(0);
  });

  it("reports a missing card as gaps, not zeros", () => {
    expect(m.gpuUtil(samples, "0000:ff:00.0")).toEqual([null, null, null]);
  });

  it("follows a graphics card by its id, not its position", () => {
    const id = samples[0].gpus[0].id;
    const reversed = samples.map((s) => ({ ...s, gpus: [...s.gpus].reverse() }));
    expect(m.gpuUtil(reversed, id)).toEqual(m.gpuUtil(samples, id));
  });

  it("keeps a card that reports no load as gaps", () => {
    const igpu = samples[0].gpus.find((g) => g.util === null)!;
    expect(m.gpuUtil(samples, igpu.id)).toEqual([null, null, null]);
    expect(m.gpuClock(samples, igpu.id).every((v) => v !== null)).toBe(true);
  });

  it("formats readings", () => {
    expect(m.percent(0)).toBe("0%");
    expect(m.percent(7.26)).toBe("7.3%");
    expect(m.percent(42.4)).toBe("42%");
    expect(m.mhz(800)).toBe("800 MHz");
    expect(m.mhz(3800)).toBe("3.8 GHz");
    expect(m.watts(4.5)).toBe("4.5 W");
    expect(m.last([1, 2, null])).toBe(0);
  });
});
