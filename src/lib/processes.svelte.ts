import { invoke } from "@tauri-apps/api/core";
import type { ProcessDetail, ProcessRow, Snapshot } from "$lib/system";

export type SortKey = "name" | "pid" | "user" | "cpu" | "memory" | "virtual_memory" | "threads";

/** Live process list, refreshed on a timer while the page is open. */
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

  private timer: ReturnType<typeof setInterval> | null = null;
  private busy = false;

  async refresh() {
    if (this.busy) return; // a slow read must not stack up behind the timer
    this.busy = true;
    try {
      this.snapshot = await invoke<Snapshot>("process_list");
      this.error = null;
      if (this.selectedPid !== null) {
        this.detail = await invoke<ProcessDetail>("process_detail", { pid: this.selectedPid });
      }
    } catch (e) {
      this.error = String(e);
    } finally {
      this.busy = false;
    }
  }

  start() {
    this.stop();
    this.refresh();
    this.timer = setInterval(() => this.live && this.refresh(), 2000);
  }

  stop() {
    if (this.timer) clearInterval(this.timer);
    this.timer = null;
  }

  async select(pid: number | null) {
    this.selectedPid = pid;
    this.detail = null;
    if (pid !== null) {
      try {
        this.detail = await invoke<ProcessDetail>("process_detail", { pid });
      } catch (e) {
        this.error = String(e);
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

  /** Rows after the search, kernel-thread filter and sort are applied. */
  get rows(): ProcessRow[] {
    const all = this.snapshot?.processes ?? [];
    const q = this.search.trim().toLowerCase();
    const list = all.filter(
      (p) =>
        (this.showKernel || !p.kernel) &&
        (!q || p.name.toLowerCase().includes(q) || p.user.toLowerCase().includes(q) || String(p.pid) === q),
    );
    const k = this.sortKey;
    const dir = this.sortDesc ? -1 : 1;
    return list.sort((a, b) => {
      const x = a[k];
      const y = b[k];
      return (typeof x === "string" ? x.localeCompare(y as string) : (x as number) - (y as number)) * dir;
    });
  }
}

export const processes = new Processes();
