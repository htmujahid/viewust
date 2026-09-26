import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

import { Poller } from "./poller";

beforeEach(() => vi.useFakeTimers());
afterEach(() => vi.useRealTimers());

describe("Poller", () => {
  it("runs at once, then on every interval", async () => {
    const task = vi.fn(async () => {});
    const poller = new Poller(task, 1000);
    poller.start();
    expect(task).toHaveBeenCalledTimes(1);
    await vi.advanceTimersByTimeAsync(3000);
    expect(task).toHaveBeenCalledTimes(4);
    poller.stop();
  });

  it("skips ticks while a run is still in flight", async () => {
    let finish!: () => void;
    const task = vi.fn(() => new Promise<void>((resolve) => (finish = resolve)));
    const poller = new Poller(task, 1000);
    poller.start();
    await vi.advanceTimersByTimeAsync(5000);
    expect(task).toHaveBeenCalledTimes(1); // never queued behind the slow one
    finish();
    await vi.advanceTimersByTimeAsync(1000);
    expect(task).toHaveBeenCalledTimes(2);
    poller.stop();
  });

  it("does nothing while inactive, and stops cleanly", async () => {
    let active = false;
    const task = vi.fn(async () => {});
    const poller = new Poller(task, 1000, () => active);
    poller.start();
    expect(task).toHaveBeenCalledTimes(1); // the first run is unconditional
    await vi.advanceTimersByTimeAsync(3000);
    expect(task).toHaveBeenCalledTimes(1);
    active = true;
    await vi.advanceTimersByTimeAsync(1000);
    expect(task).toHaveBeenCalledTimes(2);
    poller.stop();
    await vi.advanceTimersByTimeAsync(5000);
    expect(task).toHaveBeenCalledTimes(2);
  });
});
