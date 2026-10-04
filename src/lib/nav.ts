import type { IconName } from "$lib/components/ui/Icon.svelte";

export interface SidebarEntry {
  href: string;
  label: string;
  icon: IconName;
  code: string;
  /** Route prefixes that light this entry up. */
  match: string[];
}

/** The three hardware pages stand alone; everything the computer runs is behind the OS. */
export const SIDEBAR: SidebarEntry[] = [
  { href: "/devices", label: "Devices", icon: "usb", code: "01", match: ["/devices", "/device"] },
  { href: "/system", label: "System", icon: "cpu", code: "02", match: ["/system"] },
  { href: "/monitor", label: "Live monitor", icon: "chart", code: "03", match: ["/monitor"] },
  {
    href: "/os",
    label: "Operating system",
    icon: "terminal",
    code: "04",
    match: ["/os", "/processes", "/services", "/disk-usage", "/accounts"],
  },
];

/** Which sidebar entry a path belongs to, or null for the overview. */
export function activeEntry(pathname: string): string | null {
  for (const entry of SIDEBAR) {
    if (entry.match.some((p) => pathname === p || pathname.startsWith(`${p}/`))) {
      return entry.href;
    }
  }
  return null;
}
