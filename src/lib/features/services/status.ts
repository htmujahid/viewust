import type { ServiceRow } from "$lib/api/types";

export type Tone = "neutral" | "ok" | "warn" | "danger";

export function statusOf(r: Pick<ServiceRow, "active" | "sub">): { label: string; tone: Tone } {
  if (r.active === "failed") return { label: "Failed", tone: "danger" };
  if (r.active === "activating" || r.active === "deactivating" || r.active === "reloading") {
    return { label: r.active[0].toUpperCase() + r.active.slice(1), tone: "warn" };
  }
  if (r.sub === "running") return { label: "Running", tone: "ok" };
  if (r.sub === "exited") return { label: "Exited", tone: "neutral" };
  if (r.sub === "dead") return { label: "Stopped", tone: "neutral" };
  return { label: r.sub ? r.sub[0].toUpperCase() + r.sub.slice(1) : "Unknown", tone: "neutral" };
}

export function bootOf(enabled: string): { label: string; tone: Tone } {
  if (enabled.startsWith("enabled")) return { label: "On", tone: "ok" };
  if (enabled === "disabled") return { label: "Off", tone: "neutral" };
  if (enabled === "masked" || enabled === "masked-runtime")
    return { label: "Masked", tone: "warn" };
  if (enabled === "static") return { label: "Static", tone: "neutral" };
  if (enabled === "-") return { label: "—", tone: "neutral" };
  return { label: enabled[0].toUpperCase() + enabled.slice(1), tone: "neutral" };
}
