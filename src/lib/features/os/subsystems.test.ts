import { describe, expect, it } from "vitest";

import { fixtures } from "$lib/api/mock";

import {
  bootloaderFact,
  connectionFacts,
  containerFacts,
  groupFacts,
  logFacts,
  loginFacts,
  cgroupFacts,
  memoryFacts,
  mountFacts,
  networkFacts,
  packageFacts,
  processFacts,
  serviceFacts,
  summaryFacts,
  userFacts,
  vmFacts,
} from "./subsystems";

const by = (facts: { label: string; value: string; tone?: string }[], label: string) =>
  facts.find((f) => f.label === label);

describe("subsystem facts", () => {
  it("say nothing while their data is still loading", () => {
    expect(processFacts(null)).toEqual([]);
    expect(memoryFacts(null)).toEqual([]);
  });

  it("name the bootloader from its boot-chain row", () => {
    expect(bootloaderFact(fixtures.osSummary())).toEqual([{ label: "Bootloader", value: "GRUB" }]);
    expect(bootloaderFact(null)).toEqual([]);
  });

  it("pick named rows out of the summary, in order", () => {
    const s = fixtures.osSummary();
    const facts = summaryFacts(s, [
      ["Boot", "Firmware"],
      ["Boot", "No such row"],
      ["Session", "Desktop"],
    ]);
    expect(facts.map((f) => f.label)).toEqual(["Firmware", "Desktop"]);
    expect(summaryFacts(null, [["Boot", "Firmware"]])).toEqual([]);
  });

  it("sum up processes and services", () => {
    const p = processFacts(fixtures.processList(1));
    expect(by(p, "Processes")?.value).toBeTruthy();
    const s = serviceFacts(fixtures.serviceList());
    expect(by(s, "Failed")).toMatchObject({ value: "1", tone: "danger" });
  });

  it("flag heavy memory use and keep light use quiet", () => {
    const base = { available: 0, swap_total: 0, swap_used: 0, details: [] };
    expect(by(memoryFacts({ ...base, total: 100, used: 95 }), "In use")?.tone).toBe("danger");
    expect(by(memoryFacts({ ...base, total: 100, used: 30 }), "In use")?.tone).toBeUndefined();
  });

  it("shorten the controller list and admit a capped count", () => {
    const c = cgroupFacts({
      version: "v2 (unified)",
      controllers: ["cpuset", "cpu", "io", "memory", "pids", "misc"],
      groups: 5000,
      capped: true,
      top: [],
    });
    expect(by(c, "Controllers")?.value).toBe("cpuset, cpu, io, memory +2");
    expect(by(c, "Groups")?.value).toBe("over 5000");
  });

  it("count people on one card and groups on the other", () => {
    const users = userFacts(fixtures.accountList());
    expect(Number(by(users, "Accounts")?.value)).toBeGreaterThan(0);
    expect(by(users, "Groups")).toBeUndefined();
    const groups = groupFacts(fixtures.accountList());
    expect(Number(by(groups, "Groups")?.value)).toBeGreaterThan(0);
  });

  it("count mounts and name the largest", () => {
    const f = mountFacts(fixtures.filesystemList());
    expect(by(f, "Mounted")?.value).toBe("9");
    expect(by(f, "With real space")?.value).toBe("6");
    expect(by(f, "Largest")?.value).toContain("/mnt/archive");
  });

  it("flag a machine with nothing up or no route", () => {
    const n = fixtures.osNetworkMock();
    expect(by(networkFacts(n), "Interfaces up")).toMatchObject({ value: "1 of 2", tone: "ok" });
    expect(by(networkFacts({ ...n, up: 0 }), "Interfaces up")?.tone).toBe("danger");
    expect(by(networkFacts({ ...n, default_route: null }), "Internet")?.tone).toBe("warn");
    expect(networkFacts(null)).toEqual([]);
  });

  it("tell quiet logs from noisy ones and spot remote logins", () => {
    expect(by(logFacts(fixtures.osLogsMock()), "Errors this boot")?.tone).toBe("warn");
    expect(by(logFacts({ ...fixtures.osLogsMock(), errors: [] }), "Errors this boot")?.tone).toBe(
      "ok",
    );
    expect(
      logFacts({
        available: false,
        size: null,
        boots: null,
        errors: [],
        truncated: false,
        note: null,
      })[0].tone,
    ).toBe("warn");
    const logins = loginFacts(fixtures.osLoginsMock());
    expect(by(logins, "From elsewhere")).toMatchObject({ value: "1", tone: "warn" });
    expect(by(logins, "Users")?.value).toBe("2");
    expect(by(connectionFacts(fixtures.osConnectionsMock()), "Established")?.value).toBe("3");
  });

  it("tell a missing runtime from a sleeping one", () => {
    expect(
      containerFacts({ runtime: null, note: "x", running: 0, containers: [], images: [] })[0].tone,
    ).toBe("warn");
    expect(by(containerFacts(fixtures.osContainersMock()), "Running")?.value).toBe("2");
    const asleep = { ...fixtures.osContainersMock(), note: "not answering" };
    expect(by(containerFacts(asleep), "State")?.tone).toBe("warn");
    expect(by(vmFacts(fixtures.osVmsMock()), "Defined")?.value).toBe("2");
    expect(vmFacts({ managers: [], running: 0, vms: [], note: "x" })[0].tone).toBe("warn");
  });

  it("cope with a machine without a package manager", () => {
    expect(packageFacts({ manager: null, packages: [] })[0].tone).toBe("warn");
  });
});
