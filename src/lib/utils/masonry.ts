export function columnCount(width: number, minColumn: number, gap: number, max = Infinity): number {
  return Math.min(max, Math.max(1, Math.floor((width + gap) / (minColumn + gap))));
}

export function distribute<T extends { cost: number }>(items: readonly T[], count: number): T[][] {
  const lanes = Array.from({ length: Math.max(1, count) }, () => ({ height: 0, items: [] as T[] }));
  for (const item of items) {
    const shortest = lanes.reduce((a, b) => (b.height < a.height ? b : a));
    shortest.items.push(item);
    shortest.height += item.cost;
  }
  return lanes.map((l) => l.items);
}

export function sectionCost(rows: readonly { value: string }[]): number {
  const lines = rows.reduce((n, r) => {
    if (r.value.includes("\n")) return n + 1 + r.value.split("\n").length * 0.6;
    return n + (r.value.length > 64 ? 2 : 1);
  }, 0);
  return 2 + lines;
}
