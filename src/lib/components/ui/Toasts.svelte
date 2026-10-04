<script lang="ts">
  import { fly } from "svelte/transition";

  import { toasts } from "$lib/stores/toast.svelte";
</script>

<div class="toasts" aria-live="polite">
  {#each toasts.items as t (t.id)}
    <div class="toast {t.tone}" role="status" transition:fly={{ y: 12, duration: 160 }}>
      <span>{t.text}</span>
      <button aria-label="Dismiss" onclick={() => toasts.dismiss(t.id)}>×</button>
    </div>
  {/each}
</div>

<style>
  .toasts {
    position: fixed;
    right: var(--s-4);
    bottom: var(--s-4);
    z-index: 120;
    display: flex;
    flex-direction: column;
    gap: var(--s-2);
    max-width: min(420px, calc(100vw - 32px));
    pointer-events: none;
  }
  .toast {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    gap: var(--s-3);
    padding: 10px var(--s-4);
    border: 1px solid var(--border-strong);
    border-left-width: 3px;
    background: var(--surface);
    box-shadow: 0 8px 24px rgb(0 0 0 / 0.45);
    font-size: var(--fs-small);
    pointer-events: auto;
  }
  .toast.ok {
    border-left-color: var(--ok);
  }
  .toast.danger {
    border-left-color: var(--danger);
  }
  .toast.neutral {
    border-left-color: var(--accent);
  }
  button {
    border: 0;
    background: transparent;
    color: var(--text-3);
    font-size: 16px;
    line-height: 1;
    cursor: pointer;
  }
  button:hover {
    color: var(--text);
  }
</style>
