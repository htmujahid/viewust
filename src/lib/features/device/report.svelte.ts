import { api, errorMessage } from "$lib/api/client";
import type { Detail } from "$lib/api/types";

export class DeviceReport {
  rows = $state.raw<Detail[] | null>(null);
  error = $state<string | null>(null);

  private current = "";

  get pending() {
    return this.rows === null && this.error === null;
  }

  async load(id: string) {
    this.current = id;
    this.rows = null;
    this.error = null;
    try {
      const rows = await api.deviceReport(id);
      if (this.current === id) this.rows = rows;
    } catch (e) {
      if (this.current === id) this.error = errorMessage(e);
    }
  }
}
