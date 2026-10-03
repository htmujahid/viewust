import { describe, expect, it } from "vitest";

import { fixtures } from "$lib/api/mock";

import { filterGroups, filterUsers, userBadges, userMatches } from "./logic";

const accounts = fixtures.accountList();
const names = (rows: { name: string }[]) => rows.map((r) => r.name);

describe("users", () => {
  it("filters by kind of account", () => {
    const people = filterUsers(accounts.users, "people", "", "name", false);
    expect(people.every((u) => u.kind === "regular")).toBe(true);
    const system = filterUsers(accounts.users, "system", "", "name", false);
    expect(system.some((u) => u.kind === "root")).toBe(true);
    expect(filterUsers(accounts.users, "admin", "", "name", false).every((u) => u.admin)).toBe(
      true,
    );
    expect(filterUsers(accounts.users, "login", "", "name", false).every((u) => u.can_login)).toBe(
      true,
    );
  });

  it("searches name, full name, id and groups", () => {
    expect(names(filterUsers(accounts.users, "all", "talha", "name", false))).toContain("talha");
    expect(filterUsers(accounts.users, "all", "docker", "name", false).length).toBeGreaterThan(0);
    expect(filterUsers(accounts.users, "all", "1000", "name", false)[0].uid).toBe("1000");
    expect(filterUsers(accounts.users, "all", "zzzz", "name", false)).toEqual([]);
  });

  it("lists people before root and system accounts when sorted by name", () => {
    const kinds = filterUsers(accounts.users, "all", "", "name", false).map((u) => u.kind);
    const firstSystem = kinds.findIndex((k) => k !== "regular");
    expect(firstSystem).toBeGreaterThan(0);
    expect(kinds.slice(0, firstSystem).every((k) => k === "regular")).toBe(true);
    expect(kinds.slice(firstSystem).every((k) => k !== "regular")).toBe(true);
  });

  it("sorts by memory with the heaviest first", () => {
    const sorted = filterUsers(accounts.users, "all", "", "memory", true);
    expect(sorted[0].memory).toBeGreaterThanOrEqual(sorted[1].memory);
  });

  it("labels each account in plain words", () => {
    const me = accounts.users.find((u) => u.current)!;
    expect(userBadges(me).map((b) => b.label)).toContain("You");
    expect(userBadges(me).map((b) => b.label)).toContain("Admin");
    const root = accounts.users.find((u) => u.kind === "root")!;
    expect(userBadges(root).map((b) => b.label)).toContain("Root");
    expect(userMatches(root, "people")).toBe(false);
  });
});

describe("groups", () => {
  it("filters and searches by member", () => {
    expect(filterGroups(accounts.groups, "admin", "", "name", false).every((g) => g.admin)).toBe(
      true,
    );
    expect(
      filterGroups(accounts.groups, "empty", "", "name", false).every((g) => !g.members.length),
    ).toBe(true);
    expect(names(filterGroups(accounts.groups, "all", "talha", "name", false))).toContain("sudo");
  });

  it("lists admin groups first when sorted by kind", () => {
    expect(filterGroups(accounts.groups, "all", "", "kind", false)[0].admin).toBe(true);
  });
});
