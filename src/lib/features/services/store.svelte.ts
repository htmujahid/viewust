import { api, errorMessage } from "$lib/api/client";
import type { ServiceDetail, ServiceRow, ServiceSnapshot } from "$lib/api/types";
import { Poller } from "$lib/utils/poller";

export type SortKey = "name" | "state" | "enabled" | "main_pid" | "memory";
export type Filter = "all" | "running" | "failed" | "stopped" | "boot";

export const stateRank = (r: ServiceRow): number => {
  if (r.active === "failed") return 0;
  if (r.active === "activating" || r.active === "deactivating" || r.active === "reloading")
    return 1;
  if (r.sub === "running") return 2;
  if (r.sub === "exited") return 3;
  return 4;
};

export const matches = (r: ServiceRow, filter: Filter): boolean => {
  switch (filter) {
    case "running":
      return r.sub === "running";
    case "failed":
      return r.active === "failed";
    case "stopped":
      return r.active === "inactive";
    case "boot":
      return r.enabled.startsWith("enabled");
    default:
      return true;
  }
};

class Services {
  snapshot = $state.raw<ServiceSnapshot | null>(null);
  error = $state<string | null>(null);
  live = $state(true);
  search = $state("");
  filter = $state<Filter>("all");
  sortKey = $state<SortKey>("memory");
  sortDesc = $state(true);

  selectedUnit = $state<string | null>(null);
  detail = $state.raw<ServiceDetail | null>(null);

  readonly #poller = new Poller(
    () => this.#refresh(),
    5000,
    () => this.live,
  );

  start = () => this.#poller.start();
  stop = () => this.#poller.stop();

  async #refresh() {
    try {
      this.snapshot = await api.serviceList();
      this.error = null;
      if (this.selectedUnit !== null) {
        this.detail = await api.serviceDetail(this.selectedUnit);
      }
    } catch (e) {
      this.error = errorMessage(e);
    }
  }

  async select(unit: string | null) {
    this.selectedUnit = unit;
    this.detail = null;
    if (unit !== null) {
      try {
        this.detail = await api.serviceDetail(unit);
      } catch (e) {
        this.error = errorMessage(e);
      }
    }
  }

  sortBy(key: SortKey) {
    if (this.sortKey === key) this.sortDesc = !this.sortDesc;
    else {
      this.sortKey = key;
      this.sortDesc = key === "memory" || key === "main_pid";
    }
  }

  get rows(): ServiceRow[] {
    const all = this.snapshot?.services ?? [];
    const q = this.search.trim().toLowerCase();
    const list = all.filter(
      (r) =>
        matches(r, this.filter) &&
        (!q ||
          r.name.toLowerCase().includes(q) ||
          r.description.toLowerCase().includes(q) ||
          String(r.main_pid) === q),
    );
    const dir = this.sortDesc ? -1 : 1;
    const value = (r: ServiceRow): string | number => {
      switch (this.sortKey) {
        case "state":
          return stateRank(r);
        case "enabled":
          return r.enabled;
        case "main_pid":
          return r.main_pid ?? -1;
        case "memory":
          return r.memory ?? -1;
        default:
          return r.name.toLowerCase();
      }
    };
    return list.sort((a, b) => {
      const x = value(a);
      const y = value(b);
      const order = typeof x === "string" ? x.localeCompare(y as string) : x - (y as number) || 0;
      return order * dir || a.name.localeCompare(b.name);
    });
  }

  count(filter: Filter): number {
    return (this.snapshot?.services ?? []).filter((r) => matches(r, filter)).length;
  }
}

export const services = new Services();
