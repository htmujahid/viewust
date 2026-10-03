import { describe, expect, it } from "vitest";

import { fixtures } from "$lib/api/mock";

import { filterNamespaces, nsLabel } from "./logic";

const namespaces = fixtures.namespaceList().namespaces;

describe("namespaces", () => {
  it("filters by kind and searches by process", () => {
    const net = filterNamespaces(namespaces, "net", "", "processes", true);
    expect(net.length).toBeGreaterThan(0);
    expect(net.every((n) => n.kind === "net")).toBe(true);
    expect(filterNamespaces(namespaces, "all", "firefox", "kind", false).length).toBeGreaterThan(0);
  });

  it("orders by the standard kind order and then by size", () => {
    const rows = filterNamespaces(namespaces, "all", "", "kind", false);
    expect(rows[0].kind).toBe("pid");
  });

  it("names kinds in plain words and passes unknown ones through", () => {
    expect(nsLabel("net")).toBe("Network");
    expect(nsLabel("future")).toBe("future");
  });
});
