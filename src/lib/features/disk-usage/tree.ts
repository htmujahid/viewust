import type { Disk, DirectoryUsage, UsageEntry, DiskVolume } from "$lib/api/types";

export type NodeState =
  | { status: "loading" }
  | { status: "error"; message: string }
  | { status: "done"; data: DirectoryUsage };

export type NoteTone = "neutral" | "warn" | "danger";

export type TreeRow =
  | { type: "disk"; key: string; depth: number; disk: Disk; expanded: boolean }
  | {
      type: "volume";
      key: string;
      depth: number;
      volume: DiskVolume;
      /** Mounted (so it has folders) or holding other volumes */
      expandable: boolean;
      expanded: boolean;
      loading: boolean;
      mounting: boolean;
    }
  | {
      type: "entry";
      key: string;
      depth: number;
      entry: UsageEntry;
      /** This entry's share of the folder it is in, 0 to 1 */
      share: number;
      expandable: boolean;
      expanded: boolean;
      loading: boolean;
    }
  | { type: "more"; key: string; depth: number; count: number; size: number }
  | { type: "note"; key: string; depth: number; tone: NoteTone; text: string };

/**
 * Keys for what is open. Disks and volumes live in the same set as folders, so they get a
 * prefix: a folder path can look just like a device path (`/dev/sda2`).
 */
export const diskKey = (disk: Disk) => `disk:${disk.path}`;
export const volumeKey = (volume: DiskVolume) => `vol:${volume.path}`;

/** Same figure `df` prints: used out of what a user can actually fill. */
export function usedPercent(r: { used: number | null; available: number | null }): number | null {
  if (r.used === null || r.available === null) return null;
  const total = r.used + r.available;
  return total > 0 ? (r.used / total) * 100 : null;
}

/** A mounted filesystem has folders to browse. */
export const isBrowsable = (v: DiskVolume): v is DiskVolume & { mount: string } =>
  v.role === "filesystem" && v.mount !== null;

/** Every volume under these, however deeply nested. */
export function allVolumes(volumes: readonly DiskVolume[]): DiskVolume[] {
  return volumes.flatMap((v) => [v, ...allVolumes(v.children)]);
}

/** What to open the first time the list appears: every drive, and any layer that holds volumes. */
export function initiallyOpen(disks: readonly Disk[]): string[] {
  return disks.flatMap((d) => [
    diskKey(d),
    ...allVolumes(d.volumes)
      .filter((v) => v.children.length > 0)
      .map(volumeKey),
  ]);
}

/** The drives, partitions, folders and files a person can see right now, in order. */
export function flatten(
  disks: readonly Disk[],
  nodes: ReadonlyMap<string, NodeState>,
  expanded: ReadonlySet<string>,
  showFiles: boolean,
  mounting: ReadonlySet<string> = new Set(),
): TreeRow[] {
  const rows: TreeRow[] = [];

  const children = (path: string, depth: number) => {
    const state = nodes.get(path);
    const note = (suffix: string, tone: NoteTone, text: string) =>
      rows.push({ type: "note", key: `${path}#${suffix}`, depth, tone, text });
    if (!state || state.status === "loading") return note("loading", "neutral", "Measuring…");
    if (state.status === "error") return note("error", "danger", state.message);

    const { data } = state;
    const shown = data.entries.filter((e) => showFiles || e.kind === "dir");
    for (const entry of shown) visit(entry, depth, data.total);
    if (data.hidden_count > 0) {
      rows.push({
        type: "more",
        key: `${path}#more`,
        depth,
        count: data.hidden_count,
        size: data.hidden_size,
      });
    }
    if (shown.length === 0 && data.hidden_count === 0 && !data.unreadable) {
      note("empty", "neutral", showFiles ? "Empty folder" : "No folders inside");
    }
    if (data.incomplete) {
      note(
        "incomplete",
        "warn",
        "Stopped at the time limit, so these sizes are lower than the real ones. Rescan to try again.",
      );
    } else if (data.unreadable) {
      note(
        "unreadable",
        "warn",
        "Some items can't be read without administrator rights, so these sizes may be low.",
      );
    }
  };

  const visit = (entry: UsageEntry, depth: number, parentTotal: number) => {
    const expandable = entry.kind === "dir" && !entry.unreadable;
    const open = expandable && expanded.has(entry.path);
    rows.push({
      type: "entry",
      key: entry.path,
      depth,
      entry,
      share: parentTotal > 0 ? entry.size / parentTotal : 0,
      expandable,
      expanded: open,
      loading: open && nodes.get(entry.path)?.status === "loading",
    });
    if (open) children(entry.path, depth + 1);
  };

  const volumeRows = (volumes: readonly DiskVolume[], depth: number) => {
    for (const volume of volumes) {
      const browsable = isBrowsable(volume);
      const expandable = browsable || volume.children.length > 0;
      const open = expandable && expanded.has(volumeKey(volume));
      rows.push({
        type: "volume",
        key: volumeKey(volume),
        depth,
        volume,
        expandable,
        expanded: open,
        loading: open && browsable && nodes.get(volume.mount)?.status === "loading",
        mounting: mounting.has(volume.path),
      });
      if (open) {
        volumeRows(volume.children, depth + 1);
        if (browsable) children(volume.mount, depth + 1);
      }
    }
  };

  for (const disk of disks) {
    const open = expanded.has(diskKey(disk));
    rows.push({ type: "disk", key: diskKey(disk), depth: 0, disk, expanded: open });
    if (open) volumeRows(disk.volumes, 1);
  }
  return rows;
}
