import { api, errorMessage } from "$lib/api/client";
import type { Edge, Node } from "@xyflow/svelte";
import { buildInternals } from "./graph";
import type { Component, SystemInfo } from "$lib/api/types";

/** The computer's internal parts, shared by the internals page and the device page. */
class Internals {
  info = $state.raw<SystemInfo | null>(null);
  nodes = $state.raw<Node[]>([]);
  edges = $state.raw<Edge[]>([]);
  total = $state(0);
  loading = $state(false);
  error = $state<string | null>(null);
  selectedId = $state<string | null>(null);
  /** Bumped when the layout is rebuilt so the canvas remounts and re-fits. */
  scan = $state(0);

  /** Per-module memory details, once the user has allowed reading them. */
  modules = $state.raw<Component[] | null>(null);
  slots = $state<number | null>(null);
  readingMemory = $state(false);
  memoryError = $state<string | null>(null);

  private rebuild() {
    if (!this.info) return;
    const g = buildInternals(this.info, this.modules);
    this.nodes = g.nodes;
    this.edges = g.edges;
    this.total = g.total;
    this.scan++;
  }

  async load() {
    this.loading = true;
    try {
      this.info = await api.systemInfo();
      this.error = null;
      this.rebuild();
    } catch (e) {
      this.error = errorMessage(e);
    } finally {
      this.loading = false;
    }
  }

  async ensure() {
    if (!this.info && !this.loading) await this.load();
  }

  /** Asks the desktop for administrator permission, then reads each RAM module. */
  async readMemory() {
    this.readingMemory = true;
    this.memoryError = null;
    try {
      const result = await api.readMemoryModules();
      this.modules = result.modules;
      this.slots = result.slots;
      this.rebuild();
    } catch (e) {
      this.memoryError = errorMessage(e);
    } finally {
      this.readingMemory = false;
    }
  }
}

export const internals = new Internals();
