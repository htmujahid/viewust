import { describe, expect, it } from "vitest";

import type { Kind } from "$lib/illustrations/kinds";

import { monitorUrl } from "./model";

describe("watching a device live", () => {
  it("sends each kind to its monitor page", () => {
    expect(monitorUrl("cpu")).toBe("/monitor/cpu");
    expect(monitorUrl("ram")).toBe("/monitor/memory");
    expect(monitorUrl("gpu")).toBe("/monitor/gpu");
    expect(monitorUrl("monitor")).toBe("/monitor/gpu");
    for (const drive of ["nvme", "ssd", "hdd", "storage"] as Kind[]) {
      expect(monitorUrl(drive), drive).toBe("/monitor/storage");
    }
    for (const net of ["nic", "router", "internet", "wireless"] as Kind[]) {
      expect(monitorUrl(net), net).toBe("/monitor/network");
    }
    expect(monitorUrl("psu")).toBe("/monitor/power");
    for (const sound of ["audio", "soundcard", "microphone"] as Kind[]) {
      expect(monitorUrl(sound), sound).toBe("/monitor/audio");
    }
    expect(monitorUrl("computer")).toBe("/monitor");
  });

  it("stays quiet for devices nothing charts", () => {
    for (const kind of ["keyboard", "mouse", "webcam", "printer", "hub", "generic"] as Kind[]) {
      expect(monitorUrl(kind), kind).toBeNull();
    }
  });
});
