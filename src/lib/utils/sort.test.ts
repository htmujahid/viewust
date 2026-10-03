import { describe, expect, it } from "vitest";

import { sortRows } from "./sort";

const rows = [
  { name: "b", n: 2 },
  { name: "a", n: 2 },
  { name: "c", n: null },
  { name: "d", n: 9 },
];
const names = (r: typeof rows) => r.map((x) => x.name);

describe("sortRows", () => {
  it("sorts numbers both ways, breaking ties by name", () => {
    expect(
      names(
        sortRows(
          rows,
          (r) => r.n,
          true,
          (r) => r.name,
        ),
      ),
    ).toEqual(["d", "a", "b", "c"]);
    expect(
      names(
        sortRows(
          rows,
          (r) => r.n,
          false,
          (r) => r.name,
        ),
      ),
    ).toEqual(["a", "b", "d", "c"]);
  });

  it("always puts missing values last", () => {
    expect(
      names(
        sortRows(
          rows,
          (r) => r.n,
          true,
          (r) => r.name,
        ),
      ).at(-1),
    ).toBe("c");
    expect(
      names(
        sortRows(
          rows,
          (r) => r.n,
          false,
          (r) => r.name,
        ),
      ).at(-1),
    ).toBe("c");
  });

  it("sorts text without regard to input order and leaves the input alone", () => {
    const sorted = sortRows(
      rows,
      (r) => r.name,
      false,
      (r) => r.name,
    );
    expect(names(sorted)).toEqual(["a", "b", "c", "d"]);
    expect(names(rows)).toEqual(["b", "a", "c", "d"]);
  });
});
