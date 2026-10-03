<script lang="ts">
  import type { Snippet } from "svelte";

  // Keep route controls without a page heading or breadcrumb bar.
  let {
    actions,
  }: {
    back: string;
    backLabel: string;
    title?: string;
    subtitle?: string;
    actions?: Snippet;
  } = $props();
</script>

{#if actions}
  <div class="actions">{@render actions()}</div>
{/if}

<style>
  .actions {
    display: flex;
    align-items: center;
    justify-content: flex-end;
    flex-wrap: wrap;
    gap: var(--s-3);
    margin-bottom: var(--s-4);
  }
</style>
