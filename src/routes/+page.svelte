<script lang="ts">
  import { onMount } from "svelte";

  import RescanButton from "$lib/components/ui/RescanButton.svelte";
  import { hardware } from "$lib/features/devices/store.svelte";
  import MapLegend from "$lib/map/MapLegend.svelte";
  import MapToolbar from "$lib/map/MapToolbar.svelte";
  import MapView from "$lib/map/MapView.svelte";

  onMount(() => hardware.ensure());

  const count = $derived(
    hardware.total === 0
      ? "Nothing plugged in"
      : `${hardware.total} external ${hardware.total === 1 ? "device" : "devices"}`,
  );
</script>

<MapView
  bind:nodes={hardware.nodes}
  bind:edges={hardware.edges}
  bind:selectedId={hardware.selectedId}
  scan={hardware.scan}
  ready={hardware.info !== null}
  error={hardware.error}
  loadingText="Scanning…"
  errorPrefix="Couldn't read hardware:"
>
  {#snippet toolbar()}
    <MapToolbar title="Connected devices" subtitle="{count} · double-click for full details">
      <RescanButton loading={hardware.loading} onclick={() => hardware.load()} label />
    </MapToolbar>
  {/snippet}
  {#snippet legend()}
    <MapLegend
      items={[
        { label: "Wired", kind: "wired" },
        { label: "Wireless", kind: "wireless" },
      ]}
    />
  {/snippet}
</MapView>
