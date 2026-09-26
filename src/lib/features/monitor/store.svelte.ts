import { api, errorMessage } from "$lib/api/client";
import type { Sample } from "$lib/api/types";
import { Poller } from "$lib/utils/poller";

/** Samples kept for the charts: one per second, so this is the visible window. */
export const WINDOW = 60;

/** Live readings of the whole machine, polled once a second while a monitor page is open. */
class Monitor {
  samples = $state.raw<Sample[]>([]);
  error = $state<string | null>(null);
  live = $state(true);

  readonly #poller = new Poller(
    () => this.#tick(),
    1000,
    () => this.live,
  );

  get latest(): Sample | null {
    return this.samples.at(-1) ?? null;
  }

  start = () => this.#poller.start();
  stop = () => this.#poller.stop();

  async #tick() {
    try {
      const sample = await api.monitorSample();
      this.samples = [...this.samples, sample].slice(-WINDOW);
      this.error = null;
    } catch (e) {
      this.error = errorMessage(e);
    }
  }
}

export const monitor = new Monitor();
