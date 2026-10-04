<script lang="ts">
  import type { Snippet } from "svelte";

  import Page from "./Page.svelte";

  let {
    title,
    actions,
    panel,
    scroll = false,
    children,
  }: {
    title: string;
    actions?: Snippet;
    panel?: Snippet;
    /** Let the whole page scroll, for content made of several blocks rather than one table. */
    scroll?: boolean;
    children: Snippet;
  } = $props();
</script>

<div class="layout">
  <div class="content">
    <Page fill={!scroll}>
      <div class="bar">
        <h1>{title}</h1>
        {#if actions}<div class="actions">{@render actions()}</div>{/if}
      </div>
      {@render children()}
    </Page>
  </div>
  {#if panel}{@render panel()}{/if}
</div>

<style>
  .layout {
    position: relative;
    display: flex;
    height: 100%;
  }
  .content {
    flex: 1;
    min-width: 0;
  }
  .bar {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    justify-content: space-between;
    gap: var(--s-3);
    margin-bottom: var(--s-4);
  }
  h1 {
    font-family: var(--font-mono);
    font-size: 20px;
    font-weight: 700;
    text-shadow: var(--glow-text);
  }
  .actions {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: var(--s-3);
  }
</style>
