import { describe, expect, it } from "vitest";

import { niceMax } from "./scale";

describe("niceMax", () => {
  it("rounds up to a clean number with headroom", () => {
    expect(niceMax(73)).toBe(100);
    expect(niceMax(4.2e6)).toBe(5e6);
    expect(niceMax(0.3)).toBe(0.5);
    expect(niceMax(1800)).toBe(2000);
  });

  it("leaves 5% of room above the data", () => {
    expect(niceMax(100)).toBe(200); // exactly 100 would touch the top edge
    expect(niceMax(95)).toBe(100);
  });

  it("copes with nothing to plot", () => {
    expect(niceMax(0)).toBe(1);
    expect(niceMax(-3)).toBe(1);
  });
});
