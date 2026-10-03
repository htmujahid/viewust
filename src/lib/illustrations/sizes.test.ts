import { describe, expect, it } from "vitest";

import { drawings } from "./drawings";
import type { Kind } from "./kinds";
import { artBox, artSize } from "./sizes";

const area = (kind: Kind, cm?: number) => {
  const { width, height } = artSize(kind, cm);
  return width * height;
};
const longest = (kind: Kind, cm?: number) => {
  const { width, height } = artSize(kind, cm);
  return Math.max(width, height);
};

describe("artSize", () => {
  it("gives every kind a drawing, a box and a positive size", () => {
    for (const kind of Object.keys(drawings) as Kind[]) {
      const [, , w, h] = artBox(kind);
      expect(w, kind).toBeGreaterThan(0);
      expect(h, kind).toBeGreaterThan(0);
      expect(artSize(kind).width, kind).toBeGreaterThan(0);
    }
  });

  it("keeps the real-world order: bigger things are drawn bigger", () => {
    expect(longest("keyboard")).toBeGreaterThan(longest("mouse"));
    expect(longest("mouse")).toBeGreaterThan(longest("webcam"));
    expect(longest("printer")).toBeGreaterThan(longest("gamepad"));
    expect(longest("gamepad")).toBeGreaterThan(longest("wireless"));
  });

  it("draws the computer larger than any monitor, as the hub of the map", () => {
    expect(longest("computer")).toBeGreaterThan(longest("monitor", 80));
  });

  it("scales a monitor by its real width", () => {
    expect(longest("monitor", 60)).toBeGreaterThan(longest("monitor", 53));
  });

  it("never makes anything too small to see", () => {
    expect(longest("wireless")).toBeGreaterThanOrEqual(40);
    expect(longest("securitykey")).toBeGreaterThanOrEqual(40);
  });

  it("clamps absurd real-world sizes", () => {
    expect(area("monitor", 100000)).toBe(area("monitor", 120));
  });
});
