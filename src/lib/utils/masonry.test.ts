import { describe, expect, it } from "vitest";

import { columnCount, distribute, sectionCost } from "./masonry";

const item = (name: string, cost: number) => ({ name, cost });

describe("masonry", () => {
  it("fits as many columns as the width allows, and at least one", () => {
    expect(columnCount(1160, 360, 28)).toBe(3);
    expect(columnCount(760, 360, 28)).toBe(2);
    expect(columnCount(300, 360, 28)).toBe(1);
    expect(columnCount(0, 360, 28)).toBe(1);
    expect(columnCount(1160, 360, 28, 2)).toBe(2);
    expect(columnCount(300, 360, 28, 2)).toBe(1);
  });

  it("puts each item in the shortest column so none is left empty", () => {
    const lanes = distribute(
      [item("a", 6), item("b", 3), item("c", 3), item("d", 20), item("e", 2)],
      3,
    );
    expect(lanes.map((l) => l.map((i) => i.name))).toEqual([["a"], ["b", "d"], ["c", "e"]]);
  });

  it("keeps a very tall card from emptying the other columns", () => {
    const lanes = distribute([item("a", 8), item("b", 4), item("c", 5), item("tall", 24)], 3);
    expect(lanes.every((l) => l.length > 0)).toBe(true);
  });

  it("falls back to one column", () => {
    expect(distribute([item("a", 1), item("b", 1)], 0)).toEqual([[item("a", 1), item("b", 1)]]);
  });

  it("weighs long and multi-line values more than short ones", () => {
    const short = sectionCost([{ value: "x" }, { value: "y" }]);
    const long = sectionCost([{ value: "x".repeat(80) }, { value: "a\nb\nc\nd" }]);
    expect(long).toBeGreaterThan(short);
  });
});
