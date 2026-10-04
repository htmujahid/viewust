<script lang="ts">
  import { onMount } from "svelte";

  import { goto } from "$app/navigation";

  import DetailList from "$lib/components/ui/DetailList.svelte";
  import Masonry from "$lib/components/ui/Masonry.svelte";
  import RescanButton from "$lib/components/ui/RescanButton.svelte";
  import SectionShell from "$lib/components/ui/SectionShell.svelte";
  import { groupBySection } from "$lib/utils/details";
  import { sectionCost } from "$lib/utils/masonry";

  import { os } from "./store.svelte";

  /** A page made of chosen sections of the one OS summary. */
  let { title, sections }: { title: string; sections: readonly string[] } = $props();

  const summary = $derived(os.summary.data);
  const items = $derived(
    groupBySection(summary?.details ?? [])
      .filter((s) => sections.includes(s.title))
      // The page names its sections in the order it wants them told.
      .sort((a, b) => sections.indexOf(a.title) - sections.indexOf(b.title))
      .map((section) => ({
        key: section.title,
        cost: sectionCost(section.rows),
        section,
      })),
  );

  function onkeydown(e: KeyboardEvent) {
    if (e.key === "Escape") goto("/os");
  }

  onMount(os.summary.ensure);
</script>

<svelte:window {onkeydown} />

<SectionShell scroll {title}>
  {#snippet actions()}
    <RescanButton loading={os.summary.loading} onclick={os.summary.load} label />
  {/snippet}

  {#if os.summary.error && !summary}
    <p class="error">Couldn't read the operating system: {os.summary.error}</p>
  {:else if !summary}
    <div class="skeleton" style="height: 320px"></div>
  {:else}
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
