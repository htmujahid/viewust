<script lang="ts">
  import type { Snippet } from "svelte";

  /**
   * One card of the live monitor: a small title, an optional headline number,
   * facts or other content on the right, and the chart (or whatever) below.
   */
  let {
    title,
    wide = false,
    href,
    value,
    aside,
    children,
  }: {
    title: string;
    /** Span every column of the grid. */
    wide?: boolean;
    /** Show a "Details →" link to this page. */
    href?: string;
    /** The headline number. */
    value?: Snippet;
    /** Facts shown at the right of the header. */
    aside?: Snippet;
    children: Snippet;
  } = $props();
</script>

<section class="card" class:wide>
  <header>
    <div>
      <h2>{title}</h2>
      {#if value}<p class="value tabular">{@render value()}</p>{/if}
    </div>
    {#if aside}{@render aside()}{/if}
    {#if href}<a class="link" {href}>Details →</a>{/if}
  </header>
  {@render children()}
</section>

<style>
  section {
    min-width: 0;
    padding: var(--s-4);
  }
  .wide {
    grid-column: 1 / -1;
  }
  header {
    display: flex;
    flex-wrap: wrap;
    align-items: flex-start;
    justify-content: space-between;
    gap: var(--s-4);
    margin-bottom: var(--s-3);
  }
  h2 {
    color: var(--text-3);
    font-size: var(--fs-label);
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.06em;
  }
  .value {
    margin-top: 2px;
    font-size: 26px;
    font-weight: 650;
    letter-spacing: -0.01em;
  }
  .value :global(small) {
    color: var(--text-2);
    font-size: var(--fs-small);
    font-weight: 400;
    letter-spacing: 0;
  }
  .link {
    color: var(--accent);
    font-size: var(--fs-small);
    font-weight: 600;
    text-decoration: none;
    white-space: nowrap;
  }
  .link:hover {
    text-decoration: underline;
  }
</style>
