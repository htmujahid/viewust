export type SortValue = string | number | null | undefined;

export function sortRows<T>(
  rows: readonly T[],
  value: (row: T) => SortValue,
  desc: boolean,
  tiebreak: (row: T) => string,
): T[] {
  const dir = desc ? -1 : 1;
  return [...rows].sort((a, b) => {
    const x = value(a);
    const y = value(b);
    const missingX = x === null || x === undefined;
    const missingY = y === null || y === undefined;
    if (missingX || missingY) {
      if (missingX && missingY) return tiebreak(a).localeCompare(tiebreak(b));
      return missingX ? 1 : -1;
    }
    const order =
      typeof x === "string" && typeof y === "string"
        ? x.localeCompare(y)
        : (x as number) - (y as number);
    return order * dir || tiebreak(a).localeCompare(tiebreak(b));
  });
}
