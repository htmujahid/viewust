import { SvelteMap, SvelteSet } from "svelte/reactivity";

import { api, errorMessage } from "$lib/api/client";
import type { Disk, DiskDevices, DiskVolume } from "$lib/api/types";
import { toasts } from "$lib/stores/toast.svelte";

import {
  allVolumes,
  diskKey,
  flatten,
  initiallyOpen,
  isBrowsable,
  volumeKey,
  type NodeState,
  type TreeRow,
} from "./tree";

const isUnder = (path: string, folder: string) =>
  folder === "/" ? path.startsWith("/") : path === folder || path.startsWith(`${folder}/`);

class DiskUsage {
  snapshot = $state.raw<DiskDevices | null>(null);
  error = $state<string | null>(null);
  loading = $state(false);
  showFiles = $state(true);

  readonly nodes = new SvelteMap<string, NodeState>();
  readonly expanded = new SvelteSet<string>();
  /** Volumes being mounted right now, by device path. */
  readonly mounting = new SvelteSet<string>();

  #opened = false;

  get disks(): Disk[] {
    return this.snapshot?.disks ?? [];
  }

  get rows(): TreeRow[] {
    return flatten(this.disks, this.nodes, this.expanded, this.showFiles, this.mounting);
  }

  load = async () => {
    this.loading = true;
    try {
      this.snapshot = await api.diskDevices();
      this.error = null;
      // Show every drive and its partitions the first time; after that, keep what was opened.
      if (!this.#opened) {
        this.#opened = true;
        for (const key of initiallyOpen(this.snapshot.disks)) this.expanded.add(key);
      }
    } catch (e) {
      this.error = errorMessage(e);
    } finally {
      this.loading = false;
    }
  };

  /** Asks the backend how big everything in one folder is. */
  async measure(path: string, refresh = false): Promise<void> {
    this.nodes.set(path, { status: "loading" });
    try {
      this.nodes.set(path, { status: "done", data: await api.directoryUsage(path, refresh) });
    } catch (e) {
      this.nodes.set(path, { status: "error", message: errorMessage(e) });
    }
  }

  #flip(key: string): boolean {
    if (this.expanded.has(key)) {
      this.expanded.delete(key);
      return false;
    }
    this.expanded.add(key);
    return true;
  }

  /** Opens or closes whatever row was clicked. */
  toggle(row: TreeRow) {
    if (row.type === "disk") this.#flip(row.key);
    else if (row.type === "volume") {
      const v = row.volume;
      if (this.#flip(row.key) && isBrowsable(v) && !this.nodes.has(v.mount))
        void this.measure(v.mount);
    } else if (row.type === "entry" && row.expandable) {
      if (this.#flip(row.key) && !this.nodes.has(row.entry.path)) void this.measure(row.entry.path);
    }
  }

  collapseAll() {
    this.expanded.clear();
  }

  /** Mounts a filesystem so its folders can be browsed, then opens it. */
  async mount(volume: DiskVolume): Promise<void> {
    if (this.mounting.has(volume.path)) return;
    this.mounting.add(volume.path);
    try {
      const at = await api.diskMount(volume.path);
      toasts.show(`Mounted ${volume.label ?? volume.name} at ${at}`, "ok");
      await this.load();
      const now = allVolumes(this.disks.flatMap((d) => d.volumes)).find(
        (v) => v.path === volume.path,
      );
      if (now && isBrowsable(now)) {
        this.expanded.add(volumeKey(now));
        void this.measure(now.mount);
      }
    } catch (e) {
      toasts.show(errorMessage(e), "danger");
    } finally {
      this.mounting.delete(volume.path);
    }
  }

  /** Measures a folder again, and every folder opened beneath it, ignoring what was remembered. */
  async rescan(path: string): Promise<void> {
    const open = [...this.expanded]
      .filter((p) => isUnder(p, path))
      .sort((a, b) => a.length - b.length);
    for (const key of [...this.nodes.keys()]) {
      if (isUnder(key, path) && !open.includes(key)) this.nodes.delete(key);
    }
    if (!open.includes(path)) open.unshift(path);
    for (const [i, p] of open.entries()) await this.measure(p, i === 0);
  }

  async rescanAll(): Promise<void> {
    await this.load();
    const tops = allVolumes(this.disks.flatMap((d) => d.volumes)).filter(
      (v) => isBrowsable(v) && this.expanded.has(volumeKey(v)),
    );
    await Promise.all(tops.map((v) => this.rescan(v.mount!)));
  }
}

export const diskUsage = new DiskUsage();
export { diskKey, volumeKey };
