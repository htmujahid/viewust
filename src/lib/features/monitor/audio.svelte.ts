import { api, errorMessage } from "$lib/api/client";
import type { AudioSample } from "$lib/api/types";
import { Poller } from "$lib/utils/poller";

import { monitor } from "./store.svelte";

/** The audio page's own once-a-second read; it pauses with the monitor's Live toggle. */
class AudioMonitor {
  latest = $state.raw<AudioSample | null>(null);
  error = $state<string | null>(null);

  readonly #poller = new Poller(
    () => this.#refresh(),
    1000,
    () => monitor.live,
  );

  start = () => this.#poller.start();
  stop = () => this.#poller.stop();

  async #refresh() {
    try {
      this.latest = await api.audioSample();
      this.error = null;
    } catch (e) {
      this.error = errorMessage(e);
    }
  }
}

export const audioMonitor = new AudioMonitor();
