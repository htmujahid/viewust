import { invoke } from "@tauri-apps/api/core";
import type { Sample } from "$lib/system";

/** Samples kept for the charts: one per second, so this is the visible window. */
export const WINDOW = 60;

/** Live readings of the whole machine, polled once a second while the page is open. */
class Monitor {
  samples = $state.raw<Sample[]>([]);
  error = $state<string | null>(null);
  live = $state(true);

  private timer: ReturnType<typeof setInterval> | null = null;
  private busy = false;

  get latest(): Sample | null {
    return this.samples.at(-1) ?? null;
  }

  async tick() {
    if (this.busy) return; // a slow read must not queue up behind the timer
    this.busy = true;
    try {
      const sample = await invoke<Sample>("monitor_sample");
      this.samples = [...this.samples, sample].slice(-WINDOW);
      this.error = null;
    } catch (e) {
      this.error = String(e);
    } finally {
      this.busy = false;
    }
  }

  start() {
    this.stop();
    this.tick();
    this.timer = setInterval(() => this.live && this.tick(), 1000);
  }

  stop() {
    if (this.timer) clearInterval(this.timer);
    this.timer = null;
  }
}

export const monitor = new Monitor();
