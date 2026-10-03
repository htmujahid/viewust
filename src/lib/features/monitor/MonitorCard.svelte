<script lang="ts">
  import type { Snippet } from "svelte";

  let {
    title,
    wide = false,
    href,
    value,
    aside,
    children,
  }: {
    title: string;
    wide?: boolean;
    href?: string;
    value?: Snippet;
    aside?: Snippet;
    children: Snippet;
  } = $props();
</script>

<section class="card" class:wide>
  <header>
    <h2>{title}</h2>
    {#if aside}{@render aside()}{/if}
    {#if href}<a class="link" {href}>Details →</a>{/if}
  </header>
  {#if value}<div class="value tabular">{@render value()}</div>{/if}
  <div class="body">{@render children()}</div>
</section>

<style>
  section {
    display: flex;
    flex-direction: column;
    min-width: 0;
    padding: var(--s-5);
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
    flex: 1;
    min-width: 0;
    overflow-wrap: anywhere;
    color: var(--text);
    font-size: var(--fs-label);
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.08em;
  }
  h2::before {
    content: "// ";
    color: var(--accent);
  }
  .value {
    margin-bottom: var(--s-4);
    font-family: var(--font-mono);
    font-size: 30px;
    font-weight: 650;
    letter-spacing: -0.01em;
    color: var(--accent);
    text-shadow: var(--glow-text);
  }
  .value :global(small) {
    color: var(--text-2);
    font-size: var(--fs-small);
    font-weight: 400;
    letter-spacing: 0;
  }
  .link {
    color: var(--text-2);
    font-family: var(--font-mono);
    font-size: 11px;
    font-weight: 600;
    letter-spacing: 0.04em;
    text-decoration: none;
    text-transform: uppercase;
    white-space: nowrap;
  }
  .link:hover {
    color: var(--accent);
    text-shadow: var(--glow-text);
  }
  .body {
    min-width: 0;
  }
</style>
