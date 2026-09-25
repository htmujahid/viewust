<script lang="ts">
  import { onMount } from "svelte";
  import { goto } from "$app/navigation";
  import { SvelteFlow, Background, Controls, Panel } from "@xyflow/svelte";
  import "@xyflow/svelte/dist/style.css";
  import Icon from "$lib/components/Icon.svelte";
  import ThemeToggle from "$lib/components/ThemeToggle.svelte";
  import { theme } from "$lib/theme.svelte";
  import { internals } from "$lib/internals.svelte";
  import DeviceNode from "$lib/flow/DeviceNode.svelte";
  import Refit from "$lib/flow/Refit.svelte";
  import DetailsPanel from "$lib/flow/DetailsPanel.svelte";
  import type { NodeInfo } from "$lib/flow/graph";

  const nodeTypes = { device: DeviceNode };

  const selected = $derived<NodeInfo | null>(
    (internals.nodes.find((n) => n.id === internals.selectedId)?.data?.info as
      | NodeInfo
      | undefined) ?? null,
  );

  // The single "System memory" part is replaced by one per module once read.
  const canReadMemory = $derived(
    !internals.modules && internals.nodes.some((n) => n.id === "sys:ram"),
  );

  function onkeydown(e: KeyboardEvent) {
    if (e.key === "Escape" && !internals.selectedId) goto("/");
  }

  onMount(() => internals.ensure());
</script>

<svelte:window {onkeydown} />

<div class="canvas">
  <div class="flow">
    {#if internals.error}
      <p class="message error">Couldn't read the computer's parts: {internals.error}</p>
    {:else if !internals.info}
      <p class="message">Looking inside…</p>
    {:else}
      {#key internals.scan}
        <SvelteFlow
          bind:nodes={internals.nodes}
          bind:edges={internals.edges}
          {nodeTypes}
          colorMode={theme.mode}
          fitView
          fitViewOptions={{ padding: 0.12 }}
          minZoom={0.2}
          nodesConnectable={false}
          zoomOnDoubleClick={false}
          onnodeclick={({ node }) => (internals.selectedId = node.id)}
          onpaneclick={() => (internals.selectedId = null)}
          deleteKey={null}
          defaultEdgeOptions={{ type: "smoothstep" }}
        >
          <Refit trigger={!!selected} />
          <Background gap={20} />
          <Controls showLock={false} />
          <Panel position="top-left">
            <div class="card panel">
              <button class="btn" onclick={() => goto("/")} aria-label="Back to connected devices">
                <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M19 12H5M11 6l-6 6 6 6" /></svg>
                Devices
              </button>
              <div>
                <h1>Inside {internals.info.computer_name}</h1>
                <p>
                  {internals.total} components · double-click for full details
                </p>
              </div>
              {#if canReadMemory}
                <button
                  class="btn"
                  onclick={() => internals.readMemory()}
                  disabled={internals.readingMemory}
                  title="Memory modules can only be read with administrator permission. You'll be asked for your password."
                >
                  {internals.readingMemory ? "Waiting for permission…" : "Read memory modules"}
                </button>
              {:else if internals.slots !== null}
                <span class="slots">
                  {internals.modules?.length ?? 0} of {internals.slots} memory slots used
                </span>
              {/if}
              <button class="btn" onclick={() => internals.load()} disabled={internals.loading} aria-label="Rescan">
                <span class="spin-wrap" class:spin={internals.loading}>
                  <Icon name="refresh" size={16} />
                </span>
              </button>
              <button class="btn" onclick={() => goto("/processes")}>
                <Icon name="activity" size={16} />
                Processes
              </button>
              <button class="btn" onclick={() => goto("/monitor")}>
                <Icon name="chart" size={16} />
                Live
              </button>
              <ThemeToggle />
            </div>
            {#if internals.memoryError}
              <p class="warn-msg">{internals.memoryError}</p>
            {/if}
          </Panel>
          <Panel position="bottom-right">
            <div class="card legend">
              <span><i class="line wired"></i> Data</span>
              <span><i class="line memory"></i> Memory</span>
              <span><i class="line power"></i> Power</span>
            </div>
          </Panel>
        </SvelteFlow>
      {/key}
    {/if}
  </div>
  {#if selected}
    <DetailsPanel info={selected} onclose={() => (internals.selectedId = null)} />
  {/if}
</div>

<style>
  .canvas {
    display: flex;
    width: 100vw;
    height: 100vh;
  }
  .flow {
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

  .panel {
    display: flex;
    align-items: center;
    gap: var(--s-4);
    padding: var(--s-3) var(--s-4);
  }
  h1 {
    font-size: 16px;
    font-weight: 650;
  }
  p {
    color: var(--text-2);
    font-size: var(--fs-small);
  }
  .slots {
    color: var(--text-2);
    font-size: var(--fs-small);
    white-space: nowrap;
  }
  .warn-msg {
    max-width: 420px;
    margin-top: var(--s-2);
    padding: var(--s-2) var(--s-3);
    border-radius: var(--radius-sm);
    background: var(--warn-soft);
    color: var(--warn);
  }
  .spin-wrap {
    display: inline-flex;
  }
  .spin {
    animation: spin 0.9s linear infinite;
  }

  .canvas :global(.svelte-flow) {
    --xy-background-color: var(--bg);
    --xy-background-pattern-dots-color-default: var(--border);
    --xy-edge-stroke-default: var(--text-3);
    --xy-controls-button-background-color-default: var(--surface);
    --xy-controls-button-background-color-hover-default: var(--surface-2);
    --xy-controls-button-color-default: var(--text-2);
    --xy-controls-button-border-color-default: var(--border);
  }
  .canvas :global(.svelte-flow__edge.wired .svelte-flow__edge-path) {
    stroke: var(--text-3);
    stroke-width: 3;
  }
  .canvas :global(.svelte-flow__edge.memory .svelte-flow__edge-path) {
    stroke: var(--accent);
    stroke-width: 3;
  }
  .canvas :global(.svelte-flow__edge.power .svelte-flow__edge-path) {
    stroke: var(--warn);
    stroke-width: 4;
  }
  .canvas :global(.svelte-flow__handle) {
    opacity: 0;
  }
  .canvas :global(.svelte-flow__node) {
    border-radius: var(--radius);
  }

  .legend {
    display: flex;
    gap: var(--s-4);
    padding: var(--s-2) var(--s-3);
    color: var(--text-2);
    font-size: var(--fs-small);
  }
  .legend span {
    display: inline-flex;
    align-items: center;
    gap: var(--s-2);
  }
  .line {
    display: inline-block;
    width: 28px;
    height: 0;
    border-top: 3px solid var(--text-3);
  }
  .line.memory {
    border-top-color: var(--accent);
  }
  .line.power {
    border-top: 4px solid var(--warn);
  }
</style>
