import { describe, expect, it } from "vitest";

import { fixtures } from "$lib/api/mock";
import type { DirectoryUsage, UsageEntry } from "$lib/api/types";

import {
  allVolumes,
  diskKey,
  flatten,
  initiallyOpen,
  isBrowsable,
  usedPercent,
  volumeKey,
  type NodeState,
} from "./tree";

const { disks } = fixtures.diskDevices();
const disk = (name: string) => disks.find((d) => d.name === name)!;
const volume = (path: string) =>
  allVolumes(disks.flatMap((d) => d.volumes)).find((v) => v.path === path)!;

const entry = (name: string, path: string, size: number, kind: UsageEntry["kind"] = "dir") => ({
  name,
  path,
  kind,
  size,
  files: 0,
  unreadable: false,
  mount: false,
});
const usage = (path: string, entries: UsageEntry[], more: Partial<DirectoryUsage> = {}) =>
  ({
    path,
    total: entries.reduce((n, e) => n + e.size, 0),
    entries,
    hidden_count: 0,
    hidden_size: 0,
    unreadable: false,
    incomplete: false,
    took_ms: 1,
    ...more,
  }) satisfies DirectoryUsage;
const done = (data: DirectoryUsage): NodeState => ({ status: "done", data });
const keys = (rows: ReturnType<typeof flatten>) => rows.map((r) => `${r.depth}:${r.key}`);

describe("the devices", () => {
  it("lists every attached drive, including ones with nothing mounted", () => {
    expect(disks.map((d) => d.name)).toEqual(["nvme0n1", "sda", "sdb", "sdc", "Network shares"]);
    expect(disk("sda").volumes.every((v) => v.mount === null)).toBe(true);
  });

  it("only offers mounted filesystems for browsing", () => {
    expect(isBrowsable(volume("/dev/sdb2"))).toBe(true);
    expect(isBrowsable(volume("/dev/sda2"))).toBe(false);
    expect(isBrowsable(volume("/dev/nvme0n1p2"))).toBe(false);
    expect(isBrowsable(volume("/dev/nvme0n1p3"))).toBe(false);
  });

  it("finds volumes inside encrypted or LVM layers", () => {
    expect(volume("/dev/mapper/cryptroot").mount).toBe("/");
  });

  it("reads used space the way df does", () => {
    expect(usedPercent({ used: 217, available: 257 })).toBeCloseTo(45.8, 0);
    expect(usedPercent({ used: null, available: 5 })).toBeNull();
    expect(usedPercent({ used: 0, available: 0 })).toBeNull();
  });

  it("opens every drive and every layer on first sight, but no folders", () => {
    const open = initiallyOpen(disks);
    expect(open).toContain(diskKey(disk("sda")));
    expect(open).toContain(volumeKey(volume("/dev/nvme0n1p3")));
    expect(open).not.toContain(volumeKey(volume("/dev/sdb2")));
  });
});

describe("the visible tree", () => {
  const all = (open: string[] = [], nodes = new Map<string, NodeState>(), files = true) =>
    flatten(disks, nodes, new Set(open), files);

  it("shows only the drives until one is opened", () => {
    const rows = all();
    expect(rows).toHaveLength(disks.length);
    expect(rows.every((r) => r.type === "disk" && !r.expanded)).toBe(true);
  });

  it("lists a drive's partitions underneath it, one level deeper", () => {
    const rows = all([diskKey(disk("sda"))]);
    const sda = keys(rows).indexOf(`0:${diskKey(disk("sda"))}`);
    expect(keys(rows).slice(sda, sda + 3)).toEqual([
      `0:${diskKey(disk("sda"))}`,
      `1:${volumeKey(volume("/dev/sda1"))}`,
      `1:${volumeKey(volume("/dev/sda2"))}`,
    ]);
  });

  it("marks what can be mounted and doesn't let an unmounted partition expand", () => {
    const rows = all([diskKey(disk("sda"))]);
    const ntfs = rows.find((r) => r.key === volumeKey(volume("/dev/sda2")));
    expect(ntfs).toMatchObject({ type: "volume", expandable: false, expanded: false });
    expect(volume("/dev/sda2").mountable).toBe(true);
  });

  it("nests the volumes of an encrypted layer under it", () => {
    const rows = all([diskKey(disk("nvme0n1")), volumeKey(volume("/dev/nvme0n1p3"))]);
    const layer = keys(rows).indexOf(`1:${volumeKey(volume("/dev/nvme0n1p3"))}`);
    expect(keys(rows)[layer + 1]).toBe(`2:${volumeKey(volume("/dev/mapper/cryptroot"))}`);
  });

  it("says it is measuring while a mounted volume's folders load", () => {
    const sdb2 = volumeKey(volume("/dev/sdb2"));
    const rows = flatten(
      disks,
      new Map([["/mnt/archive", { status: "loading" } as NodeState]]),
      new Set([diskKey(disk("sdb")), sdb2]),
      true,
    );
    const at = rows.findIndex((r) => r.key === sdb2);
    expect(rows[at]).toMatchObject({ loading: true, expanded: true });
    expect(rows[at + 1]).toMatchObject({ type: "note", text: "Measuring…", depth: 2 });
  });

  it("lists folders under their volume, nested, with their share", () => {
    const sdb2 = volumeKey(volume("/dev/sdb2"));
    const nodes = new Map<string, NodeState>([
      [
        "/mnt/archive",
        done(
          usage("/mnt/archive", [
            entry("a", "/mnt/archive/a", 300),
            entry("b", "/mnt/archive/b", 100),
          ]),
        ),
      ],
      [
        "/mnt/archive/a",
        done(usage("/mnt/archive/a", [entry("deep", "/mnt/archive/a/deep", 300)])),
      ],
    ]);
    const rows = flatten(
      disks,
      nodes,
      new Set([diskKey(disk("sdb")), sdb2, "/mnt/archive/a"]),
      true,
    );
    const at = rows.findIndex((r) => r.key === sdb2);
    expect(keys(rows).slice(at + 1, at + 4)).toEqual([
      "2:/mnt/archive/a",
      "3:/mnt/archive/a/deep",
      "2:/mnt/archive/b",
    ]);
    expect(rows[at + 1]).toMatchObject({ share: 0.75, expanded: true });
  });

  it("hides files on request but keeps folders", () => {
    const sdb2 = volumeKey(volume("/dev/sdb2"));
    const nodes = new Map<string, NodeState>([
      [
        "/mnt/archive",
        done(
          usage("/mnt/archive", [
            entry("d", "/mnt/archive/d", 10),
            entry("f", "/mnt/archive/f", 90, "file"),
          ]),
        ),
      ],
    ]);
    const open = new Set([diskKey(disk("sdb")), sdb2]);
    expect(flatten(disks, nodes, open, true).filter((r) => r.type === "entry")).toHaveLength(2);
    expect(flatten(disks, nodes, open, false).filter((r) => r.type === "entry")).toHaveLength(1);
  });

  it("notes what was left out, empty folders, unreadable items and a stopped scan", () => {
    const sdb2 = volumeKey(volume("/dev/sdb2"));
    const open = new Set([diskKey(disk("sdb")), sdb2]);
    const last = (data: DirectoryUsage) =>
      flatten(disks, new Map([["/mnt/archive", done(data)]]), open, true).find(
        (r) => r.key.startsWith("/mnt/archive#") && !r.key.endsWith("#more"),
      );
    const withMore = flatten(
      disks,
      new Map([
        [
          "/mnt/archive",
          done(usage("/mnt/archive", [entry("a", "x", 1)], { hidden_count: 12, hidden_size: 5 })),
        ],
      ]),
      open,
      true,
    ).find((r) => r.type === "more");
    expect(withMore).toMatchObject({ count: 12, size: 5 });
    expect(last(usage("/mnt/archive", []))).toMatchObject({ text: "Empty folder" });
    expect(last(usage("/mnt/archive", [entry("a", "x", 1)], { unreadable: true }))).toMatchObject({
      tone: "warn",
    });
    expect(last(usage("/mnt/archive", [entry("a", "x", 1)], { incomplete: true }))).toMatchObject({
      tone: "warn",
      text: expect.stringContaining("time limit"),
    });
  });

  it("shows a failure where the folders would be, and won't open an unreadable folder", () => {
    const sdb2 = volumeKey(volume("/dev/sdb2"));
    const open = new Set([diskKey(disk("sdb")), sdb2, "/mnt/archive/root"]);
    const failed = flatten(
      disks,
      new Map([["/mnt/archive", { status: "error", message: "Permission denied" } as NodeState]]),
      open,
      true,
    );
    expect(failed.find((r) => r.type === "note")).toMatchObject({
      tone: "danger",
      text: "Permission denied",
    });

    const locked = { ...entry("root", "/mnt/archive/root", 4096), unreadable: true };
    const rows = flatten(
      disks,
      new Map([["/mnt/archive", done(usage("/mnt/archive", [locked]))]]),
      open,
      true,
    );
    expect(rows.find((r) => r.key === "/mnt/archive/root")).toMatchObject({
      expandable: false,
      expanded: false,
    });
  });

  it("keeps a device path and a folder with the same name apart", () => {
    expect(volumeKey(volume("/dev/sda2"))).not.toBe("/dev/sda2");
    expect(diskKey(disk("sda"))).not.toBe("/dev/sda");
  });
});
