<script lang="ts">
  import { onMount } from "svelte";

  import { goto } from "$app/navigation";

  import RescanButton from "$lib/components/ui/RescanButton.svelte";
  import { internals } from "$lib/features/internals/store.svelte";
  import MapLegend from "$lib/map/MapLegend.svelte";
  import MapToolbar from "$lib/map/MapToolbar.svelte";
  import MapView from "$lib/map/MapView.svelte";

  onMount(() => internals.ensure());

  // The single "System memory" part is replaced by one per module once read.
  const canReadMemory = $derived(
    !internals.modules && internals.nodes.some((n) => n.id === "sys:ram"),
  );

  function onkeydown(e: KeyboardEvent) {
    if (e.key === "Escape" && !internals.selectedId) goto("/");
  }
</script>

<svelte:window {onkeydown} />

<MapView
  bind:nodes={internals.nodes}
  bind:edges={internals.edges}
  bind:selectedId={internals.selectedId}
  scan={internals.scan}
  ready={internals.info !== null}
  error={internals.error}
  loadingText="Looking inside…"
  errorPrefix="Couldn't read the computer's parts:"
  fitPadding={0.12}
>
  {#snippet toolbar()}
    <MapToolbar
      title="Inside {internals.info?.computer_name}"
      subtitle="{internals.total} components · double-click for full details"
      back={{ href: "/", label: "Devices" }}
      notice={internals.memoryError}
    >
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
      <RescanButton loading={internals.loading} onclick={() => internals.load()} />
    </MapToolbar>
  {/snippet}
  {#snippet legend()}
    <MapLegend
      items={[
        { label: "Data", kind: "wired" },
        { label: "Memory", kind: "memory" },
        { label: "Power", kind: "power" },
      ]}
    />
  {/snippet}
</MapView>

<style>
  .slots {
    color: var(--text-2);
    font-size: var(--fs-small);
    white-space: nowrap;
  }
</style>
