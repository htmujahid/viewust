<script lang="ts">
  import { onMount } from "svelte";

  import { goto } from "$app/navigation";

  import DetailList from "$lib/components/ui/DetailList.svelte";
  import Masonry from "$lib/components/ui/Masonry.svelte";
  import Meter from "$lib/components/ui/Meter.svelte";
  import RescanButton from "$lib/components/ui/RescanButton.svelte";
  import StatTiles from "$lib/components/ui/StatTiles.svelte";
  import SectionShell from "$lib/components/ui/SectionShell.svelte";
  import { os } from "$lib/features/os/store.svelte";
  import { groupBySection } from "$lib/utils/details";
  import { formatBytes } from "$lib/utils/format";
  import { sectionCost } from "$lib/utils/masonry";

  const data = $derived(os.memory.data);
  const items = $derived(
    groupBySection(data?.details ?? []).map((section) => ({
      key: section.title,
      cost: sectionCost(section.rows),
      section,
    })),
  );
  const usedShare = $derived(data && data.total > 0 ? (data.used / data.total) * 100 : 0);

  function onkeydown(e: KeyboardEvent) {
    if (e.key === "Escape") goto("/os");
  }

  onMount(os.memory.ensure);
</script>

<svelte:window {onkeydown} />

<SectionShell scroll title="Memory management">
  {#snippet actions()}
    <RescanButton loading={os.memory.loading} onclick={os.memory.load} label />
  {/snippet}

  {#if os.memory.error && !data}
    <p class="error">Couldn't read the memory figures: {os.memory.error}</p>
  {:else if !data}
    <div class="skeleton" style="height: 96px; margin-bottom: 16px"></div>
    <div class="skeleton" style="height: 320px"></div>
  {:else}
    <StatTiles
      items={[
        { label: "Memory", value: formatBytes(data.total) },
        {
          label: "In use",
          value: formatBytes(data.used),
          sub: `${usedShare.toFixed(0)}%`,
          tone: usedShare >= 90 ? "danger" : usedShare >= 75 ? "warn" : undefined,
        },
        { label: "Available", value: formatBytes(data.available) },
        {
          label: "Swap",
          value: data.swap_total ? formatBytes(data.swap_used) : "None",
          sub: data.swap_total ? `of ${formatBytes(data.swap_total)}` : "no swap set up",
          tone: data.swap_total && data.swap_used / data.swap_total >= 0.5 ? "warn" : undefined,
        },
      ]}
    />
    <div class="meter"><Meter value={usedShare} /></div>

    <Masonry {items}>
      {#snippet children(item)}
        <section class="card block">
          <h2>{item.section.title}</h2>
          <DetailList rows={item.section.rows} stackAt={64} />
        </section>
      {/snippet}
    </Masonry>
  {/if}
</SectionShell>

<style>
  .meter {
    margin-bottom: var(--s-5);
  }
  .block {
    margin: 0;
    padding: var(--s-4) var(--s-5);
  }
  .block h2 {
    margin-bottom: var(--s-3);
    color: var(--accent);
    font-family: var(--font-mono);
    font-size: var(--fs-label);
    font-weight: 700;
    letter-spacing: 0.06em;
    text-transform: uppercase;
  }
  .error {
    padding: var(--s-6) 0;
    color: var(--danger);
    text-align: center;
  }
</style>
