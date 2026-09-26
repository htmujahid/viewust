<script lang="ts">
  import type { Snippet } from "svelte";
  import {
    Background,
    Controls,
    Panel,
    SvelteFlow,
    type Edge,
    type FitViewOptions,
    type Node,
  } from "@xyflow/svelte";
  import "@xyflow/svelte/dist/style.css";

  import { theme } from "$lib/stores/theme.svelte";

  import ComputerNode from "./ComputerNode.svelte";
  import DetailsPanel from "./DetailsPanel.svelte";
  import DeviceNode from "./DeviceNode.svelte";
  import type { NodeInfo } from "./model";
  import Refit from "./Refit.svelte";

  /**
   * A full-page diagram: the canvas, a floating toolbar and legend, and the
   * sidebar that opens when a node is clicked. Pages supply the data and the
   * toolbar; this owns everything about how the diagram looks and behaves.
   */
  let {
    nodes = $bindable(),
    edges = $bindable(),
    selectedId = $bindable(null),
    scan,
    ready,
    error,
    loadingText,
    errorPrefix,
    fitPadding = 0.15,
    toolbar,
    legend,
  }: {
    nodes: Node[];
    edges: Edge[];
    selectedId: string | null;
    /** Changes whenever the layout is rebuilt, so the view re-fits. */
    scan: number;
    /** False until the first data has arrived. */
    ready: boolean;
    error: string | null;
    loadingText: string;
    errorPrefix: string;
    fitPadding?: number;
    toolbar: Snippet;
    legend: Snippet;
  } = $props();

  const nodeTypes = { computer: ComputerNode, device: DeviceNode };

  /** Room kept at the top so the floating toolbar never covers the diagram. */
  const TOOLBAR_CLEARANCE = "96px";
  const padding = $derived<NonNullable<FitViewOptions["padding"]>>({
    top: TOOLBAR_CLEARANCE,
    right: fitPadding,
    bottom: fitPadding,
    left: fitPadding,
  });

  const selected = $derived(
    (nodes.find((n) => n.id === selectedId)?.data?.info as NodeInfo | undefined) ?? null,
  );
</script>

<div class="map">
  <div class="canvas">
    {#if error}
      <p class="message error">{errorPrefix} {error}</p>
    {:else if !ready}
      <p class="message">{loadingText}</p>
    {:else}
      {#key scan}
        <SvelteFlow
          bind:nodes
          bind:edges
          {nodeTypes}
          colorMode={theme.mode}
          fitView
          fitViewOptions={{ padding }}
          minZoom={0.2}
          nodesConnectable={false}
          zoomOnDoubleClick={false}
          onnodeclick={({ node }) => (selectedId = node.id)}
          onpaneclick={() => (selectedId = null)}
          deleteKey={null}
          defaultEdgeOptions={{ type: "smoothstep" }}
        >
          <Refit trigger={!!selected} {padding} />
          <Background gap={20} />
          <Controls showLock={false} />
          <Panel position="top-left">{@render toolbar()}</Panel>
          <Panel position="bottom-right">{@render legend()}</Panel>
        </SvelteFlow>
      {/key}
    {/if}
  </div>
  {#if selected}
    <DetailsPanel info={selected} onclose={() => (selectedId = null)} />
  {/if}
</div>

<style>
  .map {
    display: flex;
    width: 100vw;
    height: 100vh;
  }
  .canvas {
    flex: 1;
    min-width: 0;
    height: 100%;
  }
  .message {
    display: grid;
    place-items: center;
    height: 100%;
    color: var(--text-2);
  }
  .error {
    color: var(--danger);
  }

  /* The diagram library draws edges and handles itself; theme them here, once. */
  .canvas :global(.svelte-flow) {
    --xy-background-color: var(--bg);
    --xy-background-pattern-dots-color-default: var(--border);
    --xy-edge-stroke-default: var(--text-3);
    --xy-edge-stroke-width-default: 1.5;
    --xy-controls-button-background-color-default: var(--surface);
    --xy-controls-button-background-color-hover-default: var(--surface-2);
    --xy-controls-button-color-default: var(--text-2);
    --xy-controls-button-border-color-default: var(--border);
    --xy-node-boxshadow-selected-default: 0 0 0 2px var(--accent);
  }
  .canvas :global(.svelte-flow__node) {
    border-radius: var(--radius);
  }
  .canvas :global(.svelte-flow__handle) {
    opacity: 0;
  }

  /* line styles: see MapLegend for what each one means */
  .canvas :global(.svelte-flow__edge.wired .svelte-flow__edge-path) {
    stroke: var(--text-3);
    stroke-width: 3;
  }
  .canvas :global(.svelte-flow__edge.wireless .svelte-flow__edge-path) {
    stroke: var(--accent);
    stroke-width: 3;
    stroke-dasharray: 1 9;
    stroke-linecap: round;
  }
  .canvas :global(.svelte-flow__edge.memory .svelte-flow__edge-path) {
    stroke: var(--accent);
    stroke-width: 3;
  }
  .canvas :global(.svelte-flow__edge.power .svelte-flow__edge-path) {
    stroke: var(--warn);
    stroke-width: 4;
  }
  .canvas :global(.svelte-flow__edge.uplink .svelte-flow__edge-path) {
    stroke: var(--ok);
    stroke-width: 3;
  }
  .canvas :global(.svelte-flow__edge.uplink.down .svelte-flow__edge-path) {
    stroke: var(--danger);
    stroke-dasharray: 6 6;
  }
  .canvas :global(.svelte-flow__edge-label) {
    padding: 2px 8px;
    border: 1px solid var(--border);
    border-radius: 999px;
    background: var(--surface);
    color: var(--text-2);
    font-size: 11.5px;
    font-weight: 500;
    white-space: nowrap;
  }
</style>
