import { invoke } from "@tauri-apps/api/core";
import type { Edge, Node } from "@xyflow/svelte";
import { buildInternals } from "$lib/flow/internals";
import type { Component, MemoryModules, SystemInfo } from "$lib/system";

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
      this.info = await invoke<SystemInfo>("system_info");
      this.error = null;
      this.rebuild();
    } catch (e) {
      this.error = String(e);
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
      const result = await invoke<MemoryModules>("read_memory_modules");
      this.modules = result.modules;
      this.slots = result.slots;
      this.rebuild();
    } catch (e) {
      this.memoryError = String(e);
    } finally {
      this.readingMemory = false;
    }
  }
}

export const internals = new Internals();
