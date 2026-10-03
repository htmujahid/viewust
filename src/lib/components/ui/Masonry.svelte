<script lang="ts" generics="T extends { key: string; cost: number }">
  import type { Snippet } from "svelte";

  import { columnCount, distribute } from "$lib/utils/masonry";

  let {
    items,
    children,
    minColumn = 360,
    gap = 28,
    maxColumns = 2,
  }: {
    items: readonly T[];
    children: Snippet<[T]>;
    minColumn?: number;
    gap?: number;
    maxColumns?: number;
  } = $props();

  let width = $state(0);
  const count = $derived(columnCount(width, minColumn, gap, maxColumns));
  const lanes = $derived(distribute(items, count));
</script>

<div class="columns" bind:clientWidth={width} style:--cols={count} style:--gap="{gap}px">
  {#each lanes as lane, i (i)}
    <div class="lane">
      {#each lane as item (item.key)}
        {@render children(item)}
      {/each}
    </div>
  {/each}
</div>

<style>
  .columns {
    display: grid;
    grid-template-columns: repeat(var(--cols), minmax(0, 1fr));
    gap: var(--gap);
    align-items: start;
  }
  .lane {
    display: flex;
    flex-direction: column;
    gap: var(--gap);
    min-width: 0;
  }
</style>
