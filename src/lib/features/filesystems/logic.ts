import type { FilesystemKind, FilesystemRow } from "$lib/api/types";

export type Filter = "all" | "local" | FilesystemKind;
export type Tone = "neutral" | "ok" | "warn" | "danger";

/** How each kind of storage is named, and what the page tells the user about it. */
export const KINDS: Record<FilesystemKind, { label: string; hint: string }> = {
  disk: { label: "Internal disks", hint: "Drives inside the computer" },
  removable: { label: "Removable", hint: "USB sticks, SD cards and external drives" },
  optical: { label: "Optical", hint: "CDs, DVDs and disc images" },
  network: { label: "Network", hint: "Shares from another computer or a NAS" },
  memory: { label: "Memory", hint: "Folders kept in RAM, emptied on restart" },
  image: { label: "Packed images", hint: "Read-only compressed images, such as snaps" },
  overlay: { label: "Containers", hint: "Layered filesystems used by containers" },
  virtual: { label: "System", hint: "Kernel views, not stored on any drive" },
};

/** The order the groups appear in, real storage first. */
export const KIND_ORDER = Object.keys(KINDS) as FilesystemKind[];

const LOCAL: readonly FilesystemKind[] = ["disk", "removable", "optical"];

export const matches = (r: FilesystemRow, filter: Filter): boolean =>
  filter === "all" ? true : filter === "local" ? LOCAL.includes(r.kind) : r.kind === filter;

/** Same figure `df` prints: used out of what a user can actually fill. */
export function usedPercent(r: Pick<FilesystemRow, "used" | "available">): number | null {
  if (r.used === null || r.available === null) return null;
  const total = r.used + r.available;
  return total > 0 ? (r.used / total) * 100 : null;
}

export function healthOf(r: FilesystemRow): { label: string; tone: Tone } | null {
  const p = usedPercent(r);
  if (p === null || r.read_only || r.kind === "memory" || r.kind === "image") return null;
  if (p >= 95) return { label: "Almost full", tone: "danger" };
  if (p >= 85) return { label: "Getting full", tone: "warn" };
  return null;
}

export function search(rows: readonly FilesystemRow[], query: string): FilesystemRow[] {
  const q = query.trim().toLowerCase();
  if (!q) return [...rows];
  return rows.filter((r) =>
    [r.mount, r.source, r.fstype, r.label ?? ""].some((v) => v.toLowerCase().includes(q)),
  );
}

export const displayName = (r: FilesystemRow): string => r.label ?? r.mount;

/** Folders the system needs to keep running. The backend refuses to unmount these too. */
const SYSTEM_MOUNTS = [
  "/",
  "/boot",
  "/boot/efi",
  "/usr",
  "/var",
  "/etc",
  "/proc",
  "/sys",
  "/dev",
  "/run",
];

/** Only real storage can be unmounted from here: not memory, system views, snaps or containers. */
export const canUnmount = (r: Pick<FilesystemRow, "kind" | "mount">): boolean =>
  ["disk", "removable", "optical", "network"].includes(r.kind) && !SYSTEM_MOUNTS.includes(r.mount);

/** Memory, system and container mounts have nothing a person would browse. */
export const canOpen = (r: Pick<FilesystemRow, "kind">): boolean => r.kind !== "virtual";
