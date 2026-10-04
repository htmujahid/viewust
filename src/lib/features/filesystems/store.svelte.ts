import { api, errorMessage } from "$lib/api/client";
import type { FilesystemRow, FilesystemSnapshot } from "$lib/api/types";
import { Poller } from "$lib/utils/poller";
import { sortRows } from "$lib/utils/sort";

import { KIND_ORDER, matches, search, usedPercent, type Filter } from "./logic";

export type SortKey = "mount" | "kind" | "fstype" | "size" | "used" | "available";

class Filesystems {
  snapshot = $state.raw<FilesystemSnapshot | null>(null);
  error = $state<string | null>(null);
  live = $state(true);
  search = $state("");
  filter = $state<Filter>("all");
  sortKey = $state<SortKey>("kind");
  sortDesc = $state(false);
  selectedMount = $state<string | null>(null);

  readonly #poller = new Poller(
    () => this.#refresh(),
    5000,
    () => this.live,
  );

  start = () => this.#poller.start();
  refresh = () => this.#refresh();
  stop = () => this.#poller.stop();

  async #refresh() {
    try {
      this.snapshot = await api.filesystemList();
      this.error = null;
    } catch (e) {
      this.error = errorMessage(e);
    }
  }

  sortBy(key: SortKey) {
    if (this.sortKey === key) this.sortDesc = !this.sortDesc;
    else {
      this.sortKey = key;
      this.sortDesc = key === "size" || key === "used" || key === "available";
    }
  }

  get selected(): FilesystemRow | undefined {
    return this.snapshot?.filesystems.find((f) => f.mount === this.selectedMount);
  }

  get rows(): FilesystemRow[] {
    const all = this.snapshot?.filesystems ?? [];
    const shown = search(
      all.filter((r) => matches(r, this.filter)),
      this.search,
    );
    const value = (r: FilesystemRow) => {
      switch (this.sortKey) {
        case "kind":
          return KIND_ORDER.indexOf(r.kind);
        case "fstype":
          return r.fstype;
        case "size":
          return r.size;
        case "used":
          return usedPercent(r);
        case "available":
          return r.available;
        default:
          return r.mount;
      }
    };
    return sortRows(shown, value, this.sortDesc, (r) => r.mount);
  }

  count(filter: Filter): number {
    return (this.snapshot?.filesystems ?? []).filter((r) => matches(r, filter)).length;
  }
}

export const filesystems = new Filesystems();
