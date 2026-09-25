import { invoke } from "@tauri-apps/api/core";
import type { Edge, Node } from "@xyflow/svelte";
import { buildGraph } from "$lib/flow/graph";
import type { HardwareInfo } from "$lib/system";

/**
 * The scanned hardware, shared by the graph page and each device's own page so
 * going back and forth doesn't rescan (and keeps node positions and selection).
 */
class Hardware {
  info = $state.raw<HardwareInfo | null>(null);
  nodes = $state.raw<Node[]>([]);
  edges = $state.raw<Edge[]>([]);
  total = $state(0);
  loading = $state(false);
  error = $state<string | null>(null);
  selectedId = $state<string | null>(null);
  /** Bumped per scan so the canvas remounts and re-fits its view. */
  scan = $state(0);

  async load() {
    this.loading = true;
    try {
      const info = await invoke<HardwareInfo>("hardware_info");
      const graph = buildGraph(info);
      this.info = info;
      this.nodes = graph.nodes;
      this.edges = graph.edges;
      this.total = graph.total;
      this.scan++;
      this.error = null;
    } catch (e) {
      this.error = String(e);
    } finally {
      this.loading = false;
    }
  }

  /** Scan only if nothing has been scanned yet (e.g. a page opened directly). */
  async ensure() {
    if (!this.info && !this.loading) await this.load();
  }
}

export const hardware = new Hardware();
