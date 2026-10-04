import { goto } from "$app/navigation";

import type { GroupRow, UserRow } from "$lib/api/types";
import { processes } from "$lib/features/processes/store.svelte";
import { separator, type MenuItem } from "$lib/stores/menu.svelte";
import { copyText } from "$lib/utils/actions";

import { accounts } from "./store.svelte";

export function userMenu(u: UserRow): MenuItem[] {
  return [
    { label: "View details", onselect: () => (accounts.selectedUser = u.name) },
    {
      label: "Show their processes",
      disabled: !u.processes,
      hint: u.processes ? String(u.processes) : undefined,
      onselect: () => {
        processes.select(null);
        processes.search = u.name;
        return goto("/processes");
      },
    },
    separator,
    { label: "Copy name", onselect: () => copyText(u.name, "name") },
    {
      label: "Copy user ID",
      hint: String(u.uid),
      onselect: () => copyText(String(u.uid), "user ID"),
    },
  ];
}

export function groupMenu(g: GroupRow): MenuItem[] {
  return [
    { label: "View details", onselect: () => (accounts.selectedGroup = g.name) },
    {
      label: "Show members' accounts",
      disabled: g.members.length === 0,
      hint: g.members.length ? String(g.members.length) : undefined,
      onselect: () => {
        accounts.selectedUser = null;
        accounts.userFilter = "all";
        accounts.userSearch = g.name;
        return goto("/accounts");
      },
    },
    separator,
    { label: "Copy name", onselect: () => copyText(g.name, "name") },
    {
      label: "Copy group ID",
      hint: String(g.gid),
      onselect: () => copyText(String(g.gid), "group ID"),
    },
  ];
}
