<script lang="ts">
  import { onDestroy, onMount } from "svelte";

  import Masonry from "$lib/components/ui/Masonry.svelte";
  import Page from "$lib/components/ui/Page.svelte";
  import { hardware } from "$lib/features/devices/store.svelte";
  import { internals } from "$lib/features/internals/store.svelte";
  import { monitor } from "$lib/features/monitor/store.svelte";
  import DevicesCard from "$lib/features/overview/DevicesCard.svelte";
  import GroupCard from "$lib/features/overview/GroupCard.svelte";
  import IdentityCard from "$lib/features/overview/IdentityCard.svelte";
  import LiveStrip from "$lib/features/overview/LiveStrip.svelte";
  import { overview } from "$lib/features/overview/store.svelte";
  import { groupComponents, type Group } from "$lib/features/overview/summary";

  type Item = { key: string; cost: number; group: Group | null };

  onMount(() => {
    void hardware.ensure();
    void internals.ensure();
    void overview.load();
    monitor.start();
  });
  onDestroy(monitor.stop);

  const info = $derived(hardware.info);
  const items = $derived<Item[]>([
    ...groupComponents(internals.info?.components ?? []).map((g) => ({
      key: g.key,
      cost: g.cost,
      group: g,
    })),
    ...(info ? [{ key: "connected", cost: 9, group: null }] : []),
  ]);
</script>

<Page>
  <IdentityCard {info} sample={monitor.latest} services={overview.services} />
  <LiveStrip sample={monitor.latest} />

  {#if internals.error && !internals.info}
    <p class="error">Couldn't read the computer's parts: {internals.error}</p>
  {:else if !internals.info}
    <div class="skeleton" style="height: 320px"></div>
  {:else}
    <Masonry {items}>
      {#snippet children(item)}
        {#if item.group}
          <GroupCard group={item.group} />
        {:else if info}
          <DevicesCard {info} />
        {/if}
      {/snippet}
    </Masonry>
  {/if}
</Page>

<style>
  .error {
    padding: var(--s-6) 0;
    color: var(--danger);
    text-align: center;
  }
</style>
