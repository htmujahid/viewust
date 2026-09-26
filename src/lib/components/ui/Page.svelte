<script lang="ts">
  import type { Snippet } from "svelte";

  /** A full-height scrolling page with a centred column of content. */
  let { maxWidth = 1180, children }: { maxWidth?: number; children: Snippet } = $props();
</script>

<div class="page" style:--page-max="{maxWidth}px">
  {@render children()}
</div>

<style>
  .page {
    height: 100vh;
    overflow-y: auto;
    padding: var(--s-4) var(--s-5) var(--s-6);
    scroll-behavior: smooth;
  }
  .page > :global(*) {
    max-width: var(--page-max);
    margin-right: auto;
    margin-left: auto;
  }
</style>
