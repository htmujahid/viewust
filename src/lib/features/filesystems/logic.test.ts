import { describe, expect, it } from "vitest";

import { fixtures } from "$lib/api/mock";

import {
  canOpen,
  canUnmount,
  displayName,
  healthOf,
  KIND_ORDER,
  matches,
  search,
  usedPercent,
} from "./logic";

const rows = fixtures.filesystemList().filesystems;
const at = (mount: string) => rows.find((r) => r.mount === mount)!;

describe("filesystem logic", () => {
  it("reads used space the way df does", () => {
    expect(usedPercent(at("/"))).toBeCloseTo(45.8, 0);
    expect(usedPercent(at("/proc"))).toBeNull();
  });

  it("warns only about volumes people can fill", () => {
    expect(healthOf(at("/media/talha/Backup"))).toEqual({ label: "Almost full", tone: "danger" });
    expect(healthOf(at("/home"))).toEqual({ label: "Getting full", tone: "warn" });
    expect(healthOf(at("/"))).toBeNull();
    expect(healthOf(at("/snap/core22/1722"))).toBeNull();
    expect(healthOf(at("/run/media/talha/INSTALL"))).toBeNull();
  });

  it("filters by kind, with local meaning anything physically attached", () => {
    const kinds = (f: Parameters<typeof matches>[1]) =>
      new Set(rows.filter((r) => matches(r, f)).map((r) => r.kind));
    expect(kinds("all").size).toBe(KIND_ORDER.length);
    expect(kinds("local")).toEqual(new Set(["disk", "removable", "optical"]));
    expect(kinds("network")).toEqual(new Set(["network"]));
  });

  it("searches the mount point, source, type and label", () => {
    expect(search(rows, "nfs").map((r) => r.mount)).toEqual(["/mnt/nas"]);
    expect(search(rows, "stick").map((r) => r.mount)).toEqual(["/media/talha/STICK"]);
    expect(search(rows, "  ")).toHaveLength(rows.length);
    expect(search(rows, "no-such-thing")).toEqual([]);
  });

  it("prefers a volume's label over its path", () => {
    expect(displayName(at("/home"))).toBe("home");
    expect(displayName(at("/"))).toBe("/");
  });

  it("covers every kind the backend can send", () => {
    expect(new Set(rows.map((r) => r.kind))).toEqual(new Set(KIND_ORDER));
  });

  it("offers to unmount only drives and shares that aren't the system's own", () => {
    expect(canUnmount(at("/media/talha/STICK"))).toBe(true);
    expect(canUnmount(at("/mnt/nas"))).toBe(true);
    expect(canUnmount(at("/home"))).toBe(true);
    expect(canUnmount(at("/"))).toBe(false);
    expect(canUnmount(at("/boot/efi"))).toBe(false);
    expect(canUnmount(at("/tmp"))).toBe(false);
    expect(canUnmount(at("/snap/core22/1722"))).toBe(false);
    expect(canUnmount(at("/proc"))).toBe(false);
  });

  it("offers to open everything except kernel views", () => {
    expect(canOpen(at("/home"))).toBe(true);
    expect(canOpen(at("/tmp"))).toBe(true);
    expect(canOpen(at("/proc"))).toBe(false);
  });
});
