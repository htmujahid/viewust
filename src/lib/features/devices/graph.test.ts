import { describe, expect, it } from "vitest";

import { fixtures } from "$lib/api/mock";
import type { HardwareInfo } from "$lib/api/types";
import { artSize } from "$lib/illustrations/sizes";
import type { DeviceData } from "$lib/map/model";

import { buildGraph } from "./graph";

const info = (net = "wired"): HardwareInfo => fixtures.hardware(net);

/** Pixel rectangle each node occupies, from its position and its drawing's size. */
function rects(graph: ReturnType<typeof buildGraph>) {
  return graph.nodes.map((n) => {
    const data = n.data as Partial<DeviceData>;
    const size = n.type === "computer" ? artSize("computer") : artSize(data.kind!, data.cm);
    return { id: n.id, x: n.position.x, y: n.position.y, w: size.width, h: size.height };
  });
}

describe("buildGraph", () => {
  const graph = buildGraph(info());

  it("has the computer, every display and every peripheral, plus router and internet", () => {
    const ids = graph.nodes.map((n) => n.id);
    expect(ids).toContain("computer");
    expect(ids).toContain("display:DP-3");
    expect(ids).toContain("001-4:webcam");
    expect(ids).toContain("net:router");
    expect(ids).toContain("net:internet");
    expect(graph.nodes).toHaveLength(13);
  });

  it("gives every node a unique id", () => {
    const ids = graph.nodes.map((n) => n.id);
    expect(new Set(ids).size).toBe(ids.length);
  });

  it("only draws edges between nodes that exist", () => {
    const ids = new Set(graph.nodes.map((n) => n.id));
    for (const e of graph.edges) {
      expect(ids.has(e.source), `source of ${e.id}`).toBe(true);
      expect(ids.has(e.target), `target of ${e.id}`).toBe(true);
    }
  });

  it("never overlaps two nodes", () => {
    const r = rects(graph);
    for (const [i, a] of r.entries()) {
      for (const b of r.slice(i + 1)) {
        const apart = a.x + a.w <= b.x || b.x + b.w <= a.x || a.y + a.h <= b.y || b.y + b.h <= a.y;
        expect(apart, `${a.id} overlaps ${b.id}`).toBe(true);
      }
    }
  });

  it("links wireless devices to their receiver, not to the computer", () => {
    const toKeyboard = graph.edges.find((e) => e.target === "001-2:keyboard")!;
    expect(toKeyboard.source).toBe("001-2");
    expect(toKeyboard.class).toBe("wireless");
    expect(graph.edges.find((e) => e.target === "001-2")!.class).toBe("wired");
  });

  it("counts external devices without the router and internet", () => {
    expect(graph.total).toBe(8);
  });
});

describe("the internet connection", () => {
  const link = (net: string) => {
    const g = buildGraph(info(net));
    return {
      toRouter: g.edges.find((e) => e.target === "net:router")!,
      uplink: g.edges.find((e) => e.target === "net:internet")!,
      cloud: g.nodes.find((n) => n.id === "net:internet")!.data as DeviceData,
      router: g.nodes.find((n) => n.id === "net:router")!.data as DeviceData,
    };
  };

  it("is a cable on Ethernet", () => {
    const { toRouter, router } = link("wired");
    expect(toRouter.class).toBe("wired");
    expect(toRouter.label).toBe("Ethernet · 1 Gbit/s");
    expect(router.wireless).toBe(false);
    expect(router.signal).toBeUndefined();
  });

  it("is a radio link with signal bars on Wi-Fi", () => {
    const { toRouter, router } = link("wifi");
    expect(toRouter.class).toBe("wireless");
    expect(toRouter.animated).toBe(true);
    expect(router.signal).toBe(82);
  });

  it("shows the problem when there is no internet", () => {
    const { uplink, cloud } = link("offline");
    expect(uplink.class).toBe("uplink down");
    expect(cloud.caption).toEqual({ text: "No internet", tone: "danger" });
  });

  it("is absent when the computer has no connection at all", () => {
    const g = buildGraph({ ...info(), connection: null });
    expect(g.nodes.some((n) => n.id === "net:router")).toBe(false);
  });
});

describe("an empty map", () => {
  it("still shows the computer", () => {
    const g = buildGraph({ ...info(), peripherals: [], displays: [], connection: null });
    expect(g.nodes.map((n) => n.id)).toEqual(["computer"]);
    expect(g.edges).toEqual([]);
    expect(g.total).toBe(0);
  });
});
