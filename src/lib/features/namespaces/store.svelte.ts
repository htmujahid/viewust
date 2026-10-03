import { api, errorMessage } from "$lib/api/client";
import type { NamespaceRow, Namespaces } from "$lib/api/types";
import { Poller } from "$lib/utils/poller";

import { filterNamespaces, type NamespaceFilter } from "./logic";

class NamespacesStore {
  snapshot = $state.raw<Namespaces | null>(null);
  error = $state<string | null>(null);
  live = $state(true);
  search = $state("");
  filter = $state<NamespaceFilter>("all");
  sortKey = $state("kind");
  sortDesc = $state(false);
  selected = $state<string | null>(null);

  readonly #poller = new Poller(
    () => this.#refresh(),
    8000,
    () => this.live,
  );

  start = () => this.#poller.start();
  stop = () => this.#poller.stop();

  async #refresh() {
    try {
      this.snapshot = await api.namespaceList();
      this.error = null;
    } catch (e) {
      this.error = errorMessage(e);
    }
  }

  get rows(): NamespaceRow[] {
    return filterNamespaces(
      this.snapshot?.namespaces ?? [],
      this.filter,
      this.search,
      this.sortKey,
      this.sortDesc,
    );
  }

  count = (kind: NamespaceFilter) =>
    filterNamespaces(this.snapshot?.namespaces ?? [], kind, "", "kind", false).length;

  sortBy(key: string) {
    if (this.sortKey === key) this.sortDesc = !this.sortDesc;
    else {
      this.sortKey = key;
      this.sortDesc = key === "processes";
    }
  }

  find(key: string | null): NamespaceRow | undefined {
    return key === null ? undefined : this.snapshot?.namespaces.find((n) => keyOf(n) === key);
  }
}

export const keyOf = (n: NamespaceRow) => `${n.kind}:${n.id}`;
export const namespaces = new NamespacesStore();
