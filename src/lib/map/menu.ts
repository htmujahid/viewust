import { goto } from "$app/navigation";

import { separator, type MenuItem } from "$lib/stores/menu.svelte";
import { copyText } from "$lib/utils/actions";

import { deviceUrl, monitorUrl, tooltip, type NodeInfo } from "./model";

export function nodeMenu(info: NodeInfo, showSummary: () => void): MenuItem[] {
  const serial = info.details.find((d) => /serial/i.test(d.label))?.value;
  return [
    { label: "Show summary", onselect: showSummary },
    {
      label: info.id === "computer" ? "Open system page" : "Open full details",
      onselect: () => goto(deviceUrl(info.id)),
    },
    monitorUrl(info.kind)
      ? { label: "Watch it live", onselect: () => goto(monitorUrl(info.kind)!) }
      : separator,
    separator,
    { label: "Copy name", onselect: () => copyText(info.title, "name") },
    { label: "Copy summary", onselect: () => copyText(tooltip(info), "summary") },
    serial
      ? {
          label: "Copy serial number",
          hint: serial,
          onselect: () => copyText(serial, "serial number"),
        }
      : separator,
  ];
}
