import { api } from "$lib/api/client";
import type { ServiceOverview } from "$lib/api/types";

class Overview {
  services = $state.raw<ServiceOverview | null>(null);

  async load() {
    try {
      const snapshot = await api.serviceList();
      this.services = snapshot.available ? snapshot.overview : null;
    } catch {
      this.services = null;
    }
  }
}

export const overview = new Overview();
