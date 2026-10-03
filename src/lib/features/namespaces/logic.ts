import type { NamespaceRow } from "$lib/api/types";
import { sortRows, type SortValue } from "$lib/utils/sort";

export type NamespaceFilter = "all" | string;

const has = (text: string | null | undefined, q: string) =>
  !!text && text.toLowerCase().includes(q);

export const NS_KINDS: Record<string, { label: string; what: string }> = {
  pid: { label: "Process IDs", what: "Which processes a program can see and signal." },
  net: { label: "Network", what: "Its own network interfaces, routes and ports." },
  mnt: { label: "Mounts", what: "Which drives and folders are visible." },
  user: { label: "Users", what: "How user and group IDs map to real accounts." },
  uts: { label: "Hostname", what: "The computer name and domain name it reports." },
  ipc: { label: "Shared memory", what: "Shared memory and message queues between programs." },
  cgroup: { label: "Resource groups", what: "Which resource limits (cgroups) it can see." },
  time: { label: "Clocks", what: "Offsets applied to the system clocks it reads." },
};

export const nsLabel = (kind: string) => NS_KINDS[kind]?.label ?? kind;

export function namespaceValue(n: NamespaceRow, key: string): SortValue {
  switch (key) {
    case "id":
      return n.id;
    case "processes":
      return n.processes;
    case "scope":
      return n.current ? 0 : 1;
    default:
      return Object.keys(NS_KINDS).indexOf(n.kind);
  }
}

export function filterNamespaces(
  rows: readonly NamespaceRow[],
  filter: NamespaceFilter,
  search: string,
  sortKey: string,
  sortDesc: boolean,
): NamespaceRow[] {
  const q = search.trim().toLowerCase();
  const list = rows.filter(
    (n) =>
      (filter === "all" || n.kind === filter) &&
      (!q ||
        String(n.id) === q ||
        has(nsLabel(n.kind), q) ||
        n.sample.some((p) => has(p.name, q) || has(p.user, q) || String(p.pid) === q)),
  );
  return sortRows(
    list,
    (n) => namespaceValue(n, sortKey),
    sortDesc,
    (n) =>
      `${String(Object.keys(NS_KINDS).indexOf(n.kind)).padStart(2, "0")}-${String(n.id).padStart(12, "0")}`,
  );
}
