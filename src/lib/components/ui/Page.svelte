<script lang="ts">
  import type { Snippet } from "svelte";

  let { fill = false, children }: { fill?: boolean; children: Snippet } = $props();
</script>

<div class="page" class:fill>
  {@render children()}
</div>

<style>
  .page {
    height: 100%;
    overflow-y: auto;
    padding: var(--s-5) var(--s-5) var(--s-6);
    scroll-behavior: smooth;
  }
  .page > :global(*) {
    max-width: var(--page-max);
    margin-right: auto;
    margin-left: auto;
  }
  .fill {
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }
  .fill > :global(*) {
    flex: none;
    width: 100%;
  }
  .fill > :global(.grow) {
    flex: 0 1 auto;
    min-height: 0;
  }
  @media (max-width: 760px) {
    .page {
      padding: 20px 16px 32px;
    }
  }
</style>
