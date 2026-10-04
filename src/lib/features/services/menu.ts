import { goto } from "$app/navigation";

import { api } from "$lib/api/client";
import type { ServiceAction, ServiceRow } from "$lib/api/types";
import { processes } from "$lib/features/processes/store.svelte";
import { separator, type MenuItem } from "$lib/stores/menu.svelte";
import { copyText, perform } from "$lib/utils/actions";

import { services } from "./store.svelte";

const DONE: Record<ServiceAction, string> = {
  start: "Started",
  stop: "Stopped",
  restart: "Restarted",
  reload: "Reloaded",
  enable: "Will start at boot:",
  disable: "Won't start at boot:",
};

export function controlService(r: ServiceRow, action: ServiceAction): Promise<void> {
  const ask =
    action === "stop" || action === "disable"
      ? {
          title: `${action === "stop" ? "Stop" : "Disable"} ${r.name}?`,
          body:
            action === "stop"
              ? "Anything that relies on this service stops working until it is started again. You'll be asked for your password."
              : "It will no longer start when the computer does. You'll be asked for your password.",
          confirmLabel: action === "stop" ? "Stop" : "Disable",
          danger: true,
        }
      : undefined;
  return perform(`${DONE[action]} ${r.name}`, () => api.serviceAction(r.unit, action), {
    ask,
    after: services.refresh,
  });
}

export function serviceMenu(r: ServiceRow): MenuItem[] {
  const running = r.active === "active" || r.active === "activating";
  const toggle = r.enabled.startsWith("enabled")
    ? ({ label: "Disable at boot", onselect: () => controlService(r, "disable") } as MenuItem)
    : r.enabled === "disabled"
      ? ({ label: "Enable at boot", onselect: () => controlService(r, "enable") } as MenuItem)
      : ({ label: "Boot setting can't be changed", disabled: true } as MenuItem);
  return [
    { label: "View details", onselect: () => services.select(r.unit) },
    r.main_pid !== null
      ? {
          label: "Show main process",
          hint: `PID ${r.main_pid}`,
          onselect: () => {
            processes.search = "";
            processes.select(r.main_pid);
            return goto("/processes");
          },
        }
      : separator,
    { label: "Copy name", onselect: () => copyText(r.name, "service name") },
    { label: "Copy unit", hint: r.unit, onselect: () => copyText(r.unit, "unit name") },
    separator,
    running
      ? { label: "Restart", onselect: () => controlService(r, "restart") }
      : { label: "Start", onselect: () => controlService(r, "start") },
    { label: "Stop", danger: true, disabled: !running, onselect: () => controlService(r, "stop") },
    toggle,
  ];
}
