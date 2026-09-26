import { describe, expect, it } from "vitest";

import { fixtures } from "$lib/api/mock";
import type { DeviceData } from "$lib/map/model";
import { artSize } from "$lib/illustrations/sizes";

import { buildInternals } from "./graph";

const system = { computer_name: "pc", components: fixtures.components };

describe("buildInternals", () => {
  const g = buildInternals(system, null);

  it("lays out every component", () => {
    expect(g.nodes).toHaveLength(fixtures.components.length);
    expect(g.total).toBe(fixtures.components.length);
  });

  it("never overlaps two parts", () => {
    const r = g.nodes.map((n) => {
      const { width, height } = artSize((n.data as DeviceData).kind);
      return { id: n.id, x: n.position.x, y: n.position.y, w: width, h: height };
    });
    for (const [i, a] of r.entries()) {
      for (const b of r.slice(i + 1)) {
        const apart = a.x + a.w <= b.x || b.x + b.w <= a.x || a.y + a.h <= b.y || b.y + b.h <= a.y;
        expect(apart, `${a.id} overlaps ${b.id}`).toBe(true);
      }
    }
  });

  it("wires power from the supply to the board, and memory from the processor", () => {
    const power = g.edges.find((e) => e.source === "sys:psu")!;
    expect(power.target).toBe("sys:board");
    expect(power.class).toBe("power");
    const memory = g.edges.find((e) => e.target === "sys:ram")!;
    expect(memory.source).toBe("sys:cpu");
    expect(memory.class).toBe("memory");
  });

  it("replaces the single memory block with one part per module once they're read", () => {
    const withModules = buildInternals(system, fixtures.memoryModules.modules);
    const ids = withModules.nodes.map((n) => n.id);
    expect(ids).not.toContain("sys:ram");
    expect(ids).toContain("sys:ram:0");
    expect(ids).toContain("sys:ram:1");
    expect(
      withModules.edges.filter((e) => e.source === "sys:cpu" && e.target.startsWith("sys:ram")),
    ).toHaveLength(2);
  });

  it("copes with a machine that is missing parts", () => {
    const minimal = buildInternals({ computer_name: "pc", components: [] }, null);
    expect(minimal.nodes).toEqual([]);
    expect(minimal.edges).toEqual([]);
  });
});
