import type { Edge, Node } from "@xyflow/svelte";
import type { Component, SystemInfo } from "$lib/api/types";
import { edge, KIND_LABELS, tooltip, type DeviceData, type NodeInfo } from "$lib/map/model";
import { artSize } from "$lib/illustrations/sizes";
import type { Kind } from "$lib/illustrations/kinds";

const GAP = 28;
const COLUMN_GAP = 120;
const ROW_GAP = 70;

const toKind = (k: string): Kind => (k in KIND_LABELS ? (k as Kind) : "generic");

function info(c: Component): NodeInfo {
  const kind = toKind(c.kind);
  return {
    id: c.id,
    kind,
    label: KIND_LABELS[kind],
    title: c.name,
    subtitle: c.subtitle ?? undefined,
    details: c.details,
  };
}

/**
 * The computer's insides, laid out like an exploded diagram: the motherboard
 * in the middle, the processor above with its memory beside it, the power
 * supply and add-in cards on the left, graphics cards on the right and drives
 * underneath. Power runs from the supply to the board; everything else is data.
 *
 * `modules` replaces the single "System memory" part with one per module once
 * the user has granted permission to read them.
 */
export function buildInternals(
  system: SystemInfo,
  modules: Component[] | null,
): { nodes: Node[]; edges: Edge[]; total: number } {
  const nodes: Node[] = [];
  const edges: Edge[] = [];

  let parts = system.components;
  if (modules?.length) {
    parts = parts.flatMap((c) => (c.kind === "ram" ? modules : [c]));
  }

  const of = (...kinds: string[]) => parts.filter((c) => kinds.includes(c.kind));
  const board = of("board")[0];
  const cpu = of("cpu")[0];
  const ram = of("ram");
  const psu = of("psu")[0];
  const gpus = of("gpu");
  const cards = of("nic", "soundcard");
  const drives = of("nvme", "ssd", "hdd");

  const size = (c: Component) => artSize(toKind(c.kind));
  const stack = (items: Component[], axis: "width" | "height") =>
    items.reduce((n, c) => n + size(c)[axis], 0) + Math.max(0, items.length - 1) * GAP;

  const add = (c: Component, x: number, y: number) =>
    nodes.push({
      id: c.id,
      type: "device",
      position: { x, y },
      data: {
        kind: toKind(c.kind),
        tooltip: tooltip(info(c)),
        wireless: false,
        info: info(c),
      } satisfies DeviceData,
    });

  // The board is the origin; everything else is positioned around it.
  const bs = board ? size(board) : { width: 330, height: 330 };
  const left = -bs.width / 2;
  const right = bs.width / 2;
  const top = -bs.height / 2;
  const bottom = bs.height / 2;
  if (board) add(board, left, top);

  // right: graphics cards
  const leftColumn = [psu, ...cards].filter(Boolean) as Component[];
  const rightHeight = stack(gpus, "height");
  const leftHeight = stack(leftColumn, "height");
  const sideHalf = Math.max(rightHeight, leftHeight, bs.height) / 2;

  let y = -rightHeight / 2;
  for (const g of gpus) {
    add(g, right + COLUMN_GAP, y);
    if (board) edges.push(edge(board.id, "r", g.id, "l", false));
    y += size(g).height + GAP;
  }

  // left: power supply on top, then add-in cards, each right-aligned to the board
  y = -leftHeight / 2;
  for (const c of leftColumn) {
    const s = size(c);
    add(c, left - COLUMN_GAP - s.width, y);
    if (board) {
      const e = edge(c.id, "r", board.id, "l", false);
      edges.push(c.kind === "psu" ? { ...e, class: "power" } : e);
    }
    y += s.height + GAP;
  }

  // bottom: drives
  const driveRow = Math.max(bottom + ROW_GAP, sideHalf + 40);
  let x = -stack(drives, "width") / 2;
  for (const d of drives) {
    add(d, x, driveRow);
    if (board) edges.push(edge(board.id, "b", d.id, "t", false));
    x += size(d).width + GAP;
  }

  // top: processor, with memory modules stacked beside it
  const cs = cpu ? size(cpu) : { width: 0, height: 0 };
  const cpuY = top - ROW_GAP - cs.height;
  if (cpu) {
    add(cpu, -cs.width / 2, cpuY);
    if (board) edges.push(edge(cpu.id, "b", board.id, "t", false));
  }
  let ry = cpuY + cs.height / 2 - stack(ram, "height") / 2;
  for (const m of ram) {
    add(m, cs.width / 2 + COLUMN_GAP, ry);
    if (cpu) edges.push({ ...edge(cpu.id, "r", m.id, "l", false), class: "memory" });
    ry += size(m).height + GAP;
  }

  return { nodes, edges, total: parts.length };
}
