<script lang="ts">
  import type { Snippet } from "svelte";

  let { children }: { children: Snippet } = $props();
</script>

<div class="grid">{@render children()}</div>

<style>
  .grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(440px, 1fr));
    gap: var(--s-4);
    align-items: start;
  }
  @media (max-width: 720px) {
    .grid {
      grid-template-columns: 1fr;
    }
  }
</style>
