import type { GroupRow, UserRow } from "$lib/api/types";
import { sortRows, type SortValue } from "$lib/utils/sort";

export type UserFilter = "all" | "people" | "system" | "admin" | "login";
export type GroupFilter = "all" | "admin" | "system" | "regular" | "empty";

export const userMatches = (u: UserRow, f: UserFilter): boolean => {
  switch (f) {
    case "people":
      return u.kind === "regular";
    case "system":
      return u.kind !== "regular";
    case "admin":
      return u.admin;
    case "login":
      return u.can_login === true;
    default:
      return true;
  }
};

export const groupMatches = (g: GroupRow, f: GroupFilter): boolean => {
  switch (f) {
    case "admin":
      return g.admin;
    case "system":
      return g.kind === "system";
    case "regular":
      return g.kind === "regular";
    case "empty":
      return g.members.length === 0;
    default:
      return true;
  }
};

const has = (text: string | null | undefined, q: string) =>
  !!text && text.toLowerCase().includes(q);

export function userValue(u: UserRow, key: string): SortValue {
  switch (key) {
    case "uid":
      return Number.isNaN(Number(u.uid)) ? u.uid : Number(u.uid);
    case "group":
      return u.primary_group.toLowerCase();
    case "groups":
      return u.groups.length;
    case "shell":
      return u.shell?.toLowerCase() ?? null;
    case "processes":
      return u.processes;
    case "memory":
      return u.memory;
    default:
      return `${kindRank(u)}${u.name.toLowerCase()}`;
  }
}

const kindRank = (u: UserRow) => (u.kind === "regular" ? 0 : u.kind === "root" ? 1 : 2);

export function filterUsers(
  users: readonly UserRow[],
  filter: UserFilter,
  search: string,
  sortKey: string,
  sortDesc: boolean,
): UserRow[] {
  const q = search.trim().toLowerCase();
  const list = users.filter(
    (u) =>
      userMatches(u, filter) &&
      (!q ||
        has(u.name, q) ||
        has(u.full_name, q) ||
        u.uid === q ||
        has(u.primary_group, q) ||
        u.groups.some((g) => has(g, q))),
  );
  return sortRows(
    list,
    (u) => userValue(u, sortKey),
    sortDesc,
    (u) => u.name.toLowerCase(),
  );
}

export function groupValue(g: GroupRow, key: string): SortValue {
  switch (key) {
    case "gid":
      return Number.isNaN(Number(g.gid)) ? g.gid : Number(g.gid);
    case "kind":
      return g.admin ? 0 : g.kind === "regular" ? 1 : 2;
    case "members":
      return g.members.length;
    default:
      return g.name.toLowerCase();
  }
}

export function filterGroups(
  groups: readonly GroupRow[],
  filter: GroupFilter,
  search: string,
  sortKey: string,
  sortDesc: boolean,
): GroupRow[] {
  const q = search.trim().toLowerCase();
  const list = groups.filter(
    (g) =>
      groupMatches(g, filter) &&
      (!q || has(g.name, q) || g.gid === q || g.members.some((m) => has(m, q))),
  );
  return sortRows(
    list,
    (g) => groupValue(g, sortKey),
    sortDesc,
    (g) => g.name.toLowerCase(),
  );
}

export function userBadges(
  u: UserRow,
): { label: string; tone: "ok" | "warn" | "danger" | "neutral" }[] {
  return [
    ...(u.current ? [{ label: "You", tone: "ok" as const }] : []),
    ...(u.kind === "root" ? [{ label: "Root", tone: "danger" as const }] : []),
    ...(u.kind === "system" ? [{ label: "System", tone: "neutral" as const }] : []),
    ...(u.kind === "regular" && !u.current ? [{ label: "Person", tone: "neutral" as const }] : []),
    ...(u.admin && u.kind !== "root" ? [{ label: "Admin", tone: "warn" as const }] : []),
    ...(u.can_login === false ? [{ label: "No login", tone: "neutral" as const }] : []),
  ];
}
