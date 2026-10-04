import { api, errorMessage } from "$lib/api/client";
import type { ProcessDetail, ProcessRow, Snapshot } from "$lib/api/types";
import { Poller } from "$lib/utils/poller";

export type SortKey = "name" | "pid" | "user" | "cpu" | "memory" | "virtual_memory" | "threads";

class Processes {
  snapshot = $state.raw<Snapshot | null>(null);
  error = $state<string | null>(null);
  live = $state(true);
  search = $state("");
  showKernel = $state(false);
  sortKey = $state<SortKey>("memory");
  sortDesc = $state(true);

  selectedPid = $state<number | null>(null);
  detail = $state.raw<ProcessDetail | null>(null);

  readonly #poller = new Poller(
    () => this.#refresh(),
    2000,
    () => this.live,
  );

  start = () => this.#poller.start();
  refresh = () => this.#refresh();
  stop = () => this.#poller.stop();

  async #refresh() {
    try {
      this.snapshot = await api.processList();
      this.error = null;
      if (this.selectedPid !== null) {
        this.detail = await api.processDetail(this.selectedPid);
      }
    } catch (e) {
      this.error = errorMessage(e);
    }
  }

  async select(pid: number | null) {
    this.selectedPid = pid;
    this.detail = null;
    if (pid !== null) {
      try {
        this.detail = await api.processDetail(pid);
      } catch (e) {
        this.error = errorMessage(e);
      }
    }
  }

  sortBy(key: SortKey) {
    if (this.sortKey === key) this.sortDesc = !this.sortDesc;
    else {
      this.sortKey = key;
      this.sortDesc = key !== "name" && key !== "user";
    }
  }

  get rows(): ProcessRow[] {
    const all = this.snapshot?.processes ?? [];
    const q = this.search.trim().toLowerCase();
    const list = all.filter(
      (p) =>
        (this.showKernel || !p.kernel) &&
        (!q ||
          p.name.toLowerCase().includes(q) ||
          p.user.toLowerCase().includes(q) ||
          String(p.pid) === q),
    );
    const k = this.sortKey;
    const dir = this.sortDesc ? -1 : 1;
    return list.sort((a, b) => {
      const x = a[k];
      const y = b[k];
      return (
        (typeof x === "string" ? x.localeCompare(y as string) : (x as number) - (y as number)) * dir
      );
    });
  }
}

export const processes = new Processes();
