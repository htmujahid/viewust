import { describe, expect, it } from "vitest";

import { fixtures } from "$lib/api/mock";

import { bootOf, statusOf } from "./status";
import { matches, stateRank } from "./store.svelte";

const rows = fixtures.serviceList().services;
const named = (name: string) => rows.find((r) => r.name === name)!;

describe("service status", () => {
  it("reads a running, a finished, a stopped and a failed service", () => {
    expect(statusOf(named("ssh"))).toEqual({ label: "Running", tone: "ok" });
    expect(statusOf(named("alsa-restore"))).toEqual({ label: "Exited", tone: "neutral" });
    expect(statusOf(named("postgresql"))).toEqual({ label: "Stopped", tone: "neutral" });
    expect(statusOf(named("nginx"))).toEqual({ label: "Failed", tone: "danger" });
  });

  it("names the boot setting in plain words", () => {
    expect(bootOf("enabled")).toEqual({ label: "On", tone: "ok" });
    expect(bootOf("disabled").label).toBe("Off");
    expect(bootOf("masked").tone).toBe("warn");
    expect(bootOf("static").label).toBe("Static");
    expect(bootOf("-").label).toBe("—");
  });

  it("ranks failed services first and stopped ones last", () => {
    const ranks = (n: string) => stateRank(named(n));
    expect(ranks("nginx")).toBeLessThan(ranks("ssh"));
    expect(ranks("ssh")).toBeLessThan(ranks("alsa-restore"));
    expect(ranks("alsa-restore")).toBeLessThan(ranks("postgresql"));
  });

  it("filters by state", () => {
    expect(rows.filter((r) => matches(r, "failed")).map((r) => r.name)).toEqual(["nginx"]);
    expect(rows.filter((r) => matches(r, "running")).every((r) => r.sub === "running")).toBe(true);
    expect(rows.filter((r) => matches(r, "all"))).toHaveLength(rows.length);
    expect(rows.filter((r) => matches(r, "boot")).every((r) => r.enabled === "enabled")).toBe(true);
  });
});
