import { describe, expect, it } from "vitest";

import { fixtures } from "$lib/api/mock";

import {
  bootedAgo,
  filterEnv,
  filterModules,
  filterPackages,
  isPathList,
  packageKey,
} from "./logic";

const modules = fixtures.kernelModules();
const packages = fixtures.osPackages().packages;
const env = fixtures.osEnvironment();

describe("os filters", () => {
  it("finds modules by name or by what uses them", () => {
    expect(filterModules(modules, "nvidia").map((m) => m.name)).toEqual([
      "nvidia",
      "nvidia_modeset",
    ]);
    expect(filterModules(modules, "btusb").map((m) => m.name)).toEqual(["bluetooth"]);
    expect(filterModules(modules, "   ")).toHaveLength(modules.length);
    expect(filterModules(modules, "zzz")).toEqual([]);
  });

  it("matches every word of a package search, in any order", () => {
    expect(filterPackages(packages, "libc6 i386")).toHaveLength(1);
    expect(filterPackages(packages, "i386 libc6")).toHaveLength(1);
    expect(filterPackages(packages, "systemd 259")).toHaveLength(1);
    expect(filterPackages(packages, "amd64").length).toBeGreaterThan(5);
  });

  it("gives each architecture of a package its own key", () => {
    const libc = packages.filter((p) => p.name === "libc6");
    expect(libc).toHaveLength(2);
    expect(new Set(libc.map(packageKey)).size).toBe(2);
    expect(new Set(packages.map(packageKey)).size).toBe(packages.length);
  });

  it("searches environment names and values, but never a hidden value", () => {
    expect(filterEnv(env, "wayland").map((e) => e.key)).toEqual(["XDG_SESSION_TYPE"]);
    expect(filterEnv(env, "github").map((e) => e.key)).toEqual(["GITHUB_TOKEN"]);
    expect(env.filter((e) => e.hidden).every((e) => e.value === "")).toBe(true);
  });

  it("recognises list-style values and counts time since boot", () => {
    expect(isPathList("/usr/bin:/bin")).toBe(true);
    expect(isPathList("en_US.UTF-8")).toBe(false);
    expect(bootedAgo(1000, 4600)).toBe(3600);
    expect(bootedAgo(5000, 4600)).toBe(0);
  });
});
