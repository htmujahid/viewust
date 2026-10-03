<script lang="ts">
  import type { Snippet } from "svelte";

  let {
    notice,
    children,
  }: {
    title: string;
    subtitle: string;
    back?: { href: string; label: string };
    notice?: string | null;
    children?: Snippet;
  } = $props();
</script>

{#if children}<div class="controls">{@render children()}</div>{/if}
{#if notice}<p class="notice">{notice}</p>{/if}

<style>
  .controls {
    display: flex;
    align-items: center;
    flex-wrap: wrap;
    justify-content: flex-end;
    gap: var(--s-3);
    max-width: min(440px, calc(100vw - 100px));
  }
  .notice {
    max-width: 320px;
    margin-top: var(--s-2);
    padding: var(--s-2) var(--s-3);
    background: var(--warn-soft);
    color: var(--warn);
  }
</style>
