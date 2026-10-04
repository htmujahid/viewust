import { api } from "$lib/api/client";
import { separator, type MenuItem } from "$lib/stores/menu.svelte";
import { copyText, perform } from "$lib/utils/actions";

import { diskUsage } from "./store.svelte";
import { isBrowsable, type TreeRow } from "./tree";

const parentOf = (path: string) => path.slice(0, path.lastIndexOf("/")) || "/";

const openIn = (target: string) => perform(`Opened ${target}`, () => api.pathOpen(target));

export function treeMenu(row: TreeRow): MenuItem[] {
  switch (row.type) {
    case "disk":
      return [
        { label: row.expanded ? "Collapse" : "Expand", onselect: () => diskUsage.toggle(row) },
        separator,
        {
          label: "Copy device path",
          hint: row.disk.path,
          onselect: () => copyText(row.disk.path, "device path"),
        },
      ];

    case "volume": {
      const v = row.volume;
      return [
        row.expandable
          ? { label: row.expanded ? "Collapse" : "Expand", onselect: () => diskUsage.toggle(row) }
          : separator,
        v.mountable ? { label: "Mount to browse", onselect: () => diskUsage.mount(v) } : separator,
        ...(isBrowsable(v)
          ? [
              { label: "Open in file manager", onselect: () => openIn(v.mount) },
              { label: "Rescan", onselect: () => diskUsage.rescan(v.mount) },
            ]
          : []),
        separator,
        ...(v.mount
          ? [
              {
                label: "Copy mount point",
                hint: v.mount,
                onselect: () => copyText(v.mount!, "mount point"),
              },
            ]
          : []),
        v.path.startsWith("/dev/")
          ? {
              label: "Copy device path",
              hint: v.path,
              onselect: () => copyText(v.path, "device path"),
            }
          : separator,
      ];
    }

    case "entry": {
      const e = row.entry;
      const isFile = e.kind !== "dir";
      return [
        row.expandable
          ? { label: row.expanded ? "Collapse" : "Expand", onselect: () => diskUsage.toggle(row) }
          : separator,
        isFile
          ? { label: "Show in file manager", onselect: () => openIn(parentOf(e.path)) }
          : { label: "Open in file manager", onselect: () => openIn(e.path) },
        ...(isFile ? [{ label: "Open", onselect: () => openIn(e.path) }] : []),
        separator,
        { label: "Copy path", hint: e.path, onselect: () => copyText(e.path, "path") },
        ...(isFile
          ? []
          : [{ label: "Rescan this folder", onselect: () => diskUsage.rescan(e.path) }]),
      ];
    }

    default:
      return [];
  }
}

export const menuTitle = (row: TreeRow): string | null =>
  row.type === "disk"
    ? (row.disk.model ?? row.disk.name)
    : row.type === "volume"
      ? (row.volume.label ?? row.volume.name)
      : row.type === "entry"
        ? row.entry.path
        : null;
