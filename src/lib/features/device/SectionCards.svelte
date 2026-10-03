<script lang="ts">
  import DetailList from "$lib/components/ui/DetailList.svelte";
  import { sectionSlug, type Section } from "$lib/utils/details";

  let {
    sections,
    pending,
    error,
  }: {
    sections: readonly Section[];
    pending: boolean;
    error: string | null;
  } = $props();

  import { columnCount, distribute, sectionCost } from "$lib/utils/masonry";

  type Item =
    | { key: string; cost: number; kind: "section"; section: Section }
    | { key: string; cost: number; kind: "pending" | "error" };

  const MIN_COLUMN = 360;
  const GAP = 28;
  const MAX_COLUMNS = 2;

  let width = $state(0);
  const count = $derived(columnCount(width, MIN_COLUMN, GAP, MAX_COLUMNS));
  const items = $derived<Item[]>([
    ...sections.map((section) => ({
      key: section.title,
      cost: sectionCost(section.rows),
      kind: "section" as const,
      section,
    })),
    ...(pending ? [{ key: "pending", cost: 3, kind: "pending" as const }] : []),
    ...(error ? [{ key: "error", cost: 3, kind: "error" as const }] : []),
  ]);
  const lanes = $derived(distribute(items, count));
</script>

<div class="columns" bind:clientWidth={width} style:--cols={count}>
  {#each lanes as lane, i (i)}
    <div class="lane">
      {#each lane as item (item.key)}
        {#if item.kind === "section"}
          <section class="card block" id={sectionSlug(item.section.title)}>
            <h2>{item.section.title}</h2>
            <DetailList rows={item.section.rows} stackAt={64} />
          </section>
        {:else if item.kind === "pending"}
          <section class="card block">
            <h2>Technical details</h2>
            <p class="muted">Reading the device…</p>
          </section>
        {:else}
          <section class="card block">
            <h2>Technical details</h2>
            <p class="error">Couldn't read them: {error}</p>
          </section>
        {/if}
      {/each}
    </div>
  {/each}
</div>

<style>
  .columns {
    display: grid;
    grid-template-columns: repeat(var(--cols), minmax(0, 1fr));
    gap: var(--s-5);
    align-items: start;
  }
  .lane {
    display: flex;
    flex-direction: column;
    gap: var(--s-5);
    min-width: 0;
  }
  .block {
    padding: var(--s-5);
    scroll-margin-top: var(--s-4);
  }
  h2 {
    margin-bottom: var(--s-2);
    font-size: var(--fs-label);
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.08em;
    color: var(--text-2);
  }
  h2::before {
    content: "// ";
    color: var(--accent);
  }
  .muted {
    color: var(--text-3);
  }
  .error {
    color: var(--danger);
  }
</style>
