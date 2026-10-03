<script lang="ts">
  import type { Snippet } from "svelte";
  import { fly } from "svelte/transition";

  let {
    title,
    subtitle,
    badges,
    label,
    onclose,
    children,
  }: {
    title: string;
    subtitle?: string | null;
    badges?: Snippet;
    label: string;
    onclose: () => void;
    children: Snippet;
  } = $props();
</script>

<aside class="card panel" transition:fly={{ x: 24, duration: 180 }} aria-label={label}>
  <header>
    <div class="title">
      <h2 class="truncate" {title}>{title}</h2>
      {#if subtitle}<p>{subtitle}</p>{/if}
      {#if badges}<div class="badges">{@render badges()}</div>{/if}
    </div>
    <button class="close" onclick={onclose} aria-label="Close">
      <svg
        width="16"
        height="16"
        viewBox="0 0 24 24"
        fill="none"
        stroke="currentColor"
        stroke-width="2"
        stroke-linecap="round"><path d="M18 6 6 18M6 6l12 12" /></svg
      >
    </button>
  </header>
  <div class="body">{@render children()}</div>
</aside>

<style>
  .panel {
    display: flex;
    flex: none;
    flex-direction: column;
    align-self: stretch;
    width: 400px;
    margin: 0;
    border-width: 0 0 0 1px;
    overflow: hidden;
  }
  header {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    gap: var(--s-3);
    padding: var(--s-4);
    border-bottom: 1px solid var(--border);
  }
  .title {
    min-width: 0;
  }
  h2 {
    font-size: 17px;
    font-weight: 650;
  }
  .title p {
    color: var(--text-2);
  }
  .badges {
    display: flex;
    flex-wrap: wrap;
    gap: var(--s-2);
    margin-top: var(--s-2);
  }
  .close {
    display: grid;
    place-items: center;
    flex: none;
    width: 28px;
    height: 28px;
    border: 0;
    background: var(--surface-2);
    color: var(--text-2);
    cursor: pointer;
  }
  .close:hover {
    color: var(--text);
  }
  .body {
    flex: 1;
    overflow-y: auto;
    padding: 0 var(--s-4) var(--s-4);
  }
  .body :global(section) {
    margin-top: var(--s-4);
  }
  .body :global(h3) {
    margin-bottom: var(--s-2);
    color: var(--text-3);
    font-size: var(--fs-label);
    font-weight: 600;
    letter-spacing: 0.06em;
    text-transform: uppercase;
  }
  @media (max-width: 1050px) {
    .panel {
      position: absolute;
      right: 0;
      top: 0;
      bottom: 0;
      z-index: 10;
      width: min(400px, 100%);
      box-shadow: -12px 0 32px rgb(0 0 0 / 0.45);
    }
  }
</style>
