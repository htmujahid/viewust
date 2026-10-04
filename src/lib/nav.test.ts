import { describe, expect, it } from "vitest";

import { activeEntry, SIDEBAR } from "./nav";

describe("navigation", () => {
  it("puts the hardware pages and the operating system in the sidebar", () => {
    expect(SIDEBAR.map((e) => e.href)).toEqual([
      "/devices",
      "/system",
      "/monitor",
      "/os",
      "/health",
      "/virtualization",
    ]);
  });

  it("lights the operating system anywhere inside it", () => {
    for (const path of [
      "/os",
      "/os/kernel",
      "/os/memory",
      "/os/security",
      "/os/packages",
      "/os/environment",
      "/processes/namespaces",
      "/services",
      "/disk-usage",
      "/accounts/groups",
    ]) {
      expect(activeEntry(path), path).toBe("/os");
    }
  });

  it("lights health and virtualization on their own pages", () => {
    expect(activeEntry("/health")).toBe("/health");
    expect(activeEntry("/healthy-snacks")).toBeNull();
    expect(activeEntry("/virtualization")).toBe("/virtualization");
    expect(activeEntry("/virtualization/containers")).toBe("/virtualization");
    expect(activeEntry("/virtualization/vms")).toBe("/virtualization");
  });

  it("lights the right hardware entry on nested routes", () => {
    expect(activeEntry("/device/usb-1-2")).toBe("/devices");
    expect(activeEntry("/monitor/cpu")).toBe("/monitor");
  });

  it("leaves the overview and lookalike paths unlit", () => {
    expect(activeEntry("/")).toBeNull();
    expect(activeEntry("/osmium")).toBeNull();
  });
});
