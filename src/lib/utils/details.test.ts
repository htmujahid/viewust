import { describe, expect, it } from "vitest";

import { groupBySection, sectionSlug, sectionsToText, sortSections } from "./details";

const row = (section: string, label: string, value = "x") => ({ section, label, value });

describe("groupBySection", () => {
  it("groups rows and keeps the order each section first appears", () => {
    const sections = groupBySection([row("B", "1"), row("A", "2"), row("B", "3")]);
    expect(sections.map((s) => s.title)).toEqual(["B", "A"]);
    expect(sections[0].rows.map((r) => r.label)).toEqual(["1", "3"]);
  });

  it("handles nothing", () => {
    expect(groupBySection([])).toEqual([]);
  });
});

describe("sortSections", () => {
  const sections = groupBySection([
    row("Z", "a"),
    row("Power", "b"),
    row("Device", "c"),
    row("Y", "d"),
  ]);

  it("puts prioritised sections first, in priority order", () => {
    expect(sortSections(sections, ["Device", "Power"]).map((s) => s.title)).toEqual([
      "Device",
      "Power",
      "Z",
      "Y",
    ]);
  });

  it("leaves the rest in their original order, and does not mutate the input", () => {
    const before = sections.map((s) => s.title);
    sortSections(sections, ["Device"]);
    expect(sections.map((s) => s.title)).toEqual(before);
  });
});

describe("sectionsToText", () => {
  it("renders a readable report", () => {
    const text = sectionsToText("Webcam: C920", groupBySection([row("Device", "Product", "C920")]));
    expect(text).toBe("Webcam: C920\n\n[Device]\nProduct: C920");
  });
});

describe("sectionSlug", () => {
  it("makes a stable url fragment", () => {
    expect(sectionSlug("Interface 1 · HID report")).toBe("s-interface-1-hid-report");
  });
});
