import { api, errorMessage } from "$lib/api/client";
import type { Edge, Node } from "@xyflow/svelte";
import { buildGraph } from "./graph";
import type { HardwareInfo } from "$lib/api/types";

class Hardware {
  info = $state.raw<HardwareInfo | null>(null);
  nodes = $state.raw<Node[]>([]);
  edges = $state.raw<Edge[]>([]);
  total = $state(0);
  loading = $state(false);
  error = $state<string | null>(null);
  selectedId = $state<string | null>(null);
  scan = $state(0);

  async load() {
    this.loading = true;
    try {
      const info = await api.hardwareInfo();
      const graph = buildGraph(info);
      this.info = info;
      this.nodes = graph.nodes;
      this.edges = graph.edges;
      this.total = graph.total;
      this.scan++;
      this.error = null;
    } catch (e) {
      this.error = errorMessage(e);
    } finally {
      this.loading = false;
    }
  }

  async ensure() {
    if (!this.info && !this.loading) await this.load();
  }
}

export const hardware = new Hardware();
