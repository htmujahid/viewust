import { describe, expect, it } from "vitest";

import { formatBytes, formatCount, formatDuration, formatRate } from "./format";

describe("formatBytes", () => {
  it("uses binary units and one decimal", () => {
    expect(formatBytes(0)).toBe("0 B");
    expect(formatBytes(512)).toBe("512 B");
    expect(formatBytes(1536)).toBe("1.5 KiB");
    expect(formatBytes(1 << 30)).toBe("1.0 GiB");
  });

  it("treats negatives as empty rather than NaN", () => {
    expect(formatBytes(-5)).toBe("0 B");
  });
});

describe("formatRate", () => {
  it("uses decimal units, the way network and disk speeds are quoted", () => {
    expect(formatRate(0)).toBe("0 B/s");
    expect(formatRate(999)).toBe("999 B/s");
    expect(formatRate(1_500_000)).toBe("1.5 MB/s");
    expect(formatRate(123_456_789)).toBe("123 MB/s");
  });
});

describe("formatDuration", () => {
  it("keeps the two most significant units", () => {
    expect(formatDuration(59)).toBe("0m 59s");
    expect(formatDuration(3700)).toBe("1h 1m");
    expect(formatDuration(90_000)).toBe("1d 1h");
  });
});

describe("formatCount", () => {
  it("shortens big numbers", () => {
    expect(formatCount(0)).toBe("0");
    expect(formatCount(840)).toBe("840");
    expect(formatCount(1234)).toBe("1.2k");
    expect(formatCount(56_789)).toBe("57k");
    expect(formatCount(1_934_532)).toBe("1.9M");
  });
});
