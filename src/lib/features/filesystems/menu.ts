import { api } from "$lib/api/client";
import type { FilesystemRow } from "$lib/api/types";
import { separator, type MenuItem } from "$lib/stores/menu.svelte";
import { copyText, perform } from "$lib/utils/actions";

import { canOpen, canUnmount, displayName } from "./logic";
import { filesystems } from "./store.svelte";

export function unmountFilesystem(r: FilesystemRow): Promise<void> {
  return perform(`Unmounted ${displayName(r)}`, () => api.filesystemAction(r.mount, "unmount"), {
    ask: {
      title: `Unmount ${displayName(r)}?`,
      body: `${r.mount} will disappear from the file manager. If something is still using it, the unmount is refused.`,
      confirmLabel: "Unmount",
      danger: true,
    },
    after: filesystems.refresh,
  });
}

export function filesystemMenu(r: FilesystemRow): MenuItem[] {
  return [
    { label: "View details", onselect: () => (filesystems.selectedMount = r.mount) },
    {
      label: "Open in file manager",
      disabled: !canOpen(r),
      onselect: () => perform(`Opened ${r.mount}`, () => api.filesystemAction(r.mount, "open")),
    },
    separator,
    { label: "Copy mount point", onselect: () => copyText(r.mount, "mount point") },
    { label: "Copy source", hint: r.source, onselect: () => copyText(r.source, "source") },
    separator,
    {
      label: "Unmount",
      danger: true,
      disabled: !canUnmount(r),
      onselect: () => unmountFilesystem(r),
    },
  ];
}
