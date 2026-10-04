import { describe, expect, it } from "vitest";

import { separator, tidy, type MenuItem } from "$lib/stores/menu.svelte";

const item = (label: string): MenuItem => ({ label });

describe("menu items", () => {
  it("drops separators at the edges and doubled ones", () => {
    const out = tidy([separator, item("a"), separator, separator, item("b"), separator]);
    expect(out.map((i) => i.label || "—")).toEqual(["a", "—", "b"]);
  });

  it("keeps an all-separator list empty", () => {
    expect(tidy([separator, separator])).toEqual([]);
  });
});
