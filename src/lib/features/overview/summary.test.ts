import { describe, expect, it } from "vitest";

import { fixtures } from "$lib/api/mock";

import { displaySummary, groupComponents, internetSummary, peripheralKinds, pick } from "./summary";

const info = fixtures.hardware("wired");
const components = fixtures.components;

describe("overview summary", () => {
  it("groups the computer's parts in a fixed, sensible order", () => {
    const keys = groupComponents(components).map((g) => g.key);
    expect(keys).toEqual([
      "board",
      "cpu",
      "memory",
      "cooling",
      "graphics",
      "storage",
      "network",
      "sound",
      "power",
    ]);
  });

  it("puts every drive kind into one storage group", () => {
    const storage = groupComponents(components).find((g) => g.key === "storage")!;
    expect(storage.items).toHaveLength(3);
  });

  it("keeps parts it doesn't recognise instead of dropping them", () => {
    const odd = [...components, { ...components[0], id: "sys:x", kind: "mystery" }];
    expect(groupComponents(odd).at(-1)?.key).toBe("other");
  });

  it("weighs a group by how many parts it lists", () => {
    const groups = groupComponents(components);
    const storage = groups.find((g) => g.key === "storage")!;
    const cpu = groups.find((g) => g.key === "cpu")!;
    expect(storage.cost).toBeGreaterThan(cpu.cost);
  });

  it("reads one detail by section and label", () => {
    expect(pick([{ section: "S", label: "L", value: "v" }], "S", "L")).toBe("v");
    expect(pick([], "S", "L")).toBeNull();
  });

  it("counts connected devices by kind with plurals", () => {
    const lines = peripheralKinds(info.peripherals);
    expect(lines.length).toBeGreaterThan(0);
    expect(
      peripheralKinds([
        { ...info.peripherals[0], kind: "mouse" },
        { ...info.peripherals[0], kind: "mouse" },
      ]),
    ).toEqual(["2 mice"]);
    expect(peripheralKinds([])).toEqual([]);
  });

  it("summarises the internet connection and displays", () => {
    expect(internetSummary(info)?.ok).toBe(true);
    expect(internetSummary({ ...info, connection: null })).toBeNull();
    expect(displaySummary(info)[0]).toContain("1920×1080");
  });
});
