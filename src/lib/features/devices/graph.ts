import type { Edge, Node } from "@xyflow/svelte";

import type { HardwareInfo, Peripheral } from "$lib/api/types";
import { artSize } from "$lib/illustrations/sizes";
import type { Kind } from "$lib/illustrations/kinds";

import {
  edge,
  KIND_LABELS,
  tooltip,
  type ComputerData,
  type DeviceData,
  type NodeInfo,
} from "$lib/map/model";

const LEFT: Kind[] = ["keyboard", "mouse", "gamepad", "wireless"];
const RIGHT: Kind[] = ["webcam", "audio", "microphone"];

const GAP = 28;
const COMPUTER = artSize("computer");
const COMPUTER_W = COMPUTER.width;
const COMPUTER_H = COMPUTER.height;
const MIN_COLUMN_GAP = 150;
const ROW_GAP = 70;

interface Entry {
  id: string;
  kind: Kind;
  wireless: boolean;
  info: NodeInfo;
  children: Entry[];
}

function toEntry(p: Peripheral, children: Peripheral[], via?: string): Entry {
  const kind = (p.kind in KIND_LABELS ? p.kind : "generic") as Kind;
  return {
    id: p.id,
    kind,
    wireless: p.wireless,
    info: {
      id: p.id,
      kind,
      label: KIND_LABELS[kind],
      title: p.name,
      subtitle: p.manufacturer ?? undefined,
      connection: p.wireless ? "Wireless" : "Wired",
      via,
      details: p.details,
    },
    children: children.map((c) => toEntry(c, [], p.name)),
  };
}

export function buildGraph(info: HardwareInfo): {
  nodes: Node[];
  edges: Edge[];
  total: number;
} {
  const nodes: Node[] = [];
  const edges: Edge[] = [];

  const byParent = new Map<string, Peripheral[]>();
  for (const p of info.peripherals) {
    if (p.via) byParent.set(p.via, [...(byParent.get(p.via) ?? []), p]);
  }
  const roots = info.peripherals
    .filter((p) => !p.via)
    .map((p) => toEntry(p, byParent.get(p.id) ?? []));

  const left = roots.filter(
    (e) => LEFT.includes(e.kind) && (e.kind !== "wireless" || e.children.length),
  );
  const right = roots.filter((e) => RIGHT.includes(e.kind));
  const bottom = roots.filter((e) => !left.includes(e) && !right.includes(e));

  const size = (e: Entry) => artSize(e.kind);
  const sum = (values: number[]) =>
    values.reduce((a, b) => a + b, 0) + Math.max(0, values.length - 1) * GAP;

  const place = (e: Entry, x: number, y: number, wirelessLink: boolean) =>
    nodes.push({
      id: e.id,
      type: "device",
      position: { x, y },
      data: {
        kind: e.kind,
        tooltip: tooltip(e.info),
        wireless: wirelessLink,
        info: e.info,
      } satisfies DeviceData,
    });

  const cx = COMPUTER_W / 2;

  const blockHeight = (e: Entry) =>
    Math.max(size(e).height, sum(e.children.map((c) => size(c).height)));
  const rightHeight = sum(right.map((e) => size(e).height));
  const leftHeight = sum(left.map(blockHeight));
  const rowOffset = Math.max(COMPUTER_H / 2 + ROW_GAP, Math.max(leftHeight, rightHeight) / 2 + 40);

  const displays = info.displays.map((d, i) => {
    const id = `display:${d.connector ?? i}`;
    return { d, id, ...artSize("monitor", d.width_cm ?? undefined) };
  });
  const topWidth = sum(displays.map((x) => x.width));
  let x = cx - topWidth / 2;
  for (const { d, id, width, height } of displays) {
    nodes.push({
      id,
      type: "device",
      position: { x, y: -rowOffset - height },
      data: {
        kind: "monitor",
        cm: d.width_cm ?? undefined,
        tooltip: `Display · ${d.name} · ${d.width} × ${d.height}${d.primary ? " · Primary" : ""}`,
        wireless: false,
        info: {
          id,
          kind: "monitor",
          label: KIND_LABELS.monitor,
          title: d.name,
          subtitle: `${d.width} × ${d.height}`,
          connection: "Wired",
          details: d.details,
        },
      } satisfies DeviceData,
    });
    edges.push(edge("computer", "t", id, "b", false));
    x += width + GAP;
  }

  const bottomWidth = sum(bottom.map((e) => size(e).width));
  x = cx - bottomWidth / 2;
  for (const e of bottom) {
    place(e, x, rowOffset, false);
    edges.push(edge("computer", "b", e.id, "t", false));
    x += size(e).width + GAP;
  }

  const overhang = Math.max(topWidth, bottomWidth) / 2 - COMPUTER_W / 2;
  const columnGap = Math.max(MIN_COLUMN_GAP, overhang + 40);

  let y = -rightHeight / 2;
  for (const e of right) {
    place(e, COMPUTER_W + columnGap, y, false);
    edges.push(edge("computer", "r", e.id, "l", false));
    y += size(e).height + GAP;
  }

  y = -leftHeight / 2;
  for (const e of left) {
    const block = blockHeight(e);
    const { width, height } = size(e);
    const ex = -columnGap - width;
    place(e, ex, y + (block - height) / 2, false);
    edges.push(edge("computer", "l", e.id, "r", false));

    let cy = y + (block - sum(e.children.map((c) => size(c).height))) / 2;
    for (const c of e.children) {
      const { width: cw, height: ch } = size(c);
      place(c, ex - columnGap * 0.7 - cw, cy, true);
      edges.push(edge(e.id, "l", c.id, "r", true));
      cy += ch + GAP;
    }
    y += block + GAP;
  }

  if (info.connection) {
    const c = info.connection;
    const router = artSize("router");
    const cloud = artSize("internet");
    const wifi = c.kind === "wifi";
    const linkY = -COMPUTER_H / 2 + COMPUTER_H * 0.85;
    const routerTop = linkY - router.height / 2;
    const x = COMPUTER_W + Math.max(columnGap, 280);

    const mediaBottom = -rightHeight / 2 + rightHeight;
    const shift = Math.min(0, routerTop - GAP - mediaBottom);
    if (shift < 0) {
      for (const n of nodes) {
        if (right.some((e) => e.id === n.id))
          n.position = { ...n.position, y: n.position.y + shift };
      }
    }

    const online = c.connectivity === "full" || c.connectivity === "unknown";
    const routerInfo: NodeInfo = {
      id: "net:router",
      kind: "router",
      label: wifi ? "Wi-Fi router" : "Router",
      title: c.router_name,
      subtitle: c.link_label,
      connection: wifi ? "Wireless" : "Wired",
      details: c.router_details,
    };
    const internetInfo: NodeInfo = {
      id: "net:internet",
      kind: "internet",
      label: "Internet",
      title: online ? "Online" : c.connectivity === "portal" ? "Sign-in needed" : "No internet",
      subtitle: `via ${c.interface}`,
      details: c.internet_details,
    };

    nodes.push({
      id: "net:router",
      type: "device",
      position: { x, y: routerTop },
      data: {
        kind: "router",
        tooltip: tooltip(routerInfo),
        wireless: wifi,
        signal: wifi ? (c.signal ?? undefined) : undefined,
        info: routerInfo,
      } satisfies DeviceData,
    });
    nodes.push({
      id: "net:internet",
      type: "device",
      position: { x: x + router.width + 150, y: linkY - cloud.height / 2 },
      data: {
        kind: "internet",
        tooltip: tooltip(internetInfo),
        wireless: false,
        caption: online
          ? { text: "Online", tone: "ok" }
          : c.connectivity === "portal"
            ? { text: "Sign-in needed", tone: "warn" }
            : { text: "No internet", tone: "danger" },
        info: internetInfo,
      } satisfies DeviceData,
    });

    edges.push({
      ...edge("computer", "r2", "net:router", "l", wifi),
      type: "default",
      class: wifi ? "wireless" : "wired",
      label: c.link_label,
    });
    edges.push({
      ...edge("net:router", "r", "net:internet", "l", false),
      type: "default",
      class: online ? "uplink" : "uplink down",
      label: online ? undefined : "No route out",
    });
  }

  nodes.push({
    id: "computer",
    type: "computer",
    position: { x: 0, y: -COMPUTER_H / 2 },
    data: {
      name: info.computer_name,
      info: {
        id: "computer",
        kind: "computer",
        label: KIND_LABELS.computer,
        title: info.computer_name,
        details: info.computer_details,
      },
    } satisfies ComputerData,
  });

  return { nodes, edges, total: info.displays.length + roots.length };
}
