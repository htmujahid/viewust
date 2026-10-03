import { api, errorMessage } from "$lib/api/client";
import type { Sample } from "$lib/api/types";
import { Poller } from "$lib/utils/poller";

export const WINDOW = 60;

export interface Energy {
  cpu: number;
  gpus: Record<string, number>;
  seconds: number;
}

const MAX_GAP_SECONDS = 5;

class Monitor {
  samples = $state.raw<Sample[]>([]);
  error = $state<string | null>(null);
  live = $state(true);
  energy = $state<Energy>({ cpu: 0, gpus: {}, seconds: 0 });

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

  #accumulate(prev: Sample | undefined, next: Sample) {
    if (!prev) return;
    const seconds = Math.min((next.t - prev.t) / 1000, MAX_GAP_SECONDS);
    if (seconds <= 0) return;
    const wh = (watts: number | null) => ((watts ?? 0) * seconds) / 3600;
    const gpus = { ...this.energy.gpus };
    for (const g of next.gpus) gpus[g.id] = (gpus[g.id] ?? 0) + wh(g.power);
    this.energy = {
      cpu: this.energy.cpu + wh(next.power.cpu_watts),
      gpus,
      seconds: this.energy.seconds + seconds,
    };
  }

  async #tick() {
    try {
      const sample = await api.monitorSample();
      this.#accumulate(this.samples.at(-1), sample);
      this.samples = [...this.samples, sample].slice(-WINDOW);
      this.error = null;
    } catch (e) {
      this.error = errorMessage(e);
    }
  }
}

export const monitor = new Monitor();
