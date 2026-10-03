<script lang="ts">
  import { onMount } from "svelte";
  import { getCurrentWindow } from "@tauri-apps/api/window";

  let maximized = $state(false);

  const win = () => {
    try {
      return getCurrentWindow();
    } catch {
      return null;
    }
  };
  const run = (action: (w: NonNullable<ReturnType<typeof win>>) => Promise<unknown>) => () => {
    const w = win();
    if (w) action(w).catch(() => {});
  };

  onMount(() => {
    const w = win();
    if (!w) return;
    const sync = () =>
      w
        .isMaximized()
        .then((m) => (maximized = m))
        .catch(() => {});
    sync();
    const off = w.onResized(sync);
    return () => off.then((fn) => fn()).catch(() => {});
  });
</script>

<header class="titlebar" data-tauri-drag-region>
  <span class="title" data-tauri-drag-region>
    <b>VIEWUST</b><i data-tauri-drag-region>// live hardware map</i>
  </span>
  <div class="controls">
    <button aria-label="Minimize" title="Minimize" onclick={run((w) => w.minimize())}>
      <svg viewBox="0 0 10 10" aria-hidden="true"><path d="M1 5h8" /></svg>
    </button>
    <button
      aria-label={maximized ? "Restore" : "Maximize"}
      title={maximized ? "Restore" : "Maximize"}
      onclick={run((w) => w.toggleMaximize())}
    >
      <svg viewBox="0 0 10 10" aria-hidden="true">
        {#if maximized}
          <path d="M2.5 2.5V1h6.5v6.5H7.5M1 2.5h6.5V9H1z" />
        {:else}
          <path d="M1.5 1.5h7v7h-7z" />
        {/if}
      </svg>
    </button>
    <button class="close" aria-label="Close" title="Close" onclick={run((w) => w.close())}>
      <svg viewBox="0 0 10 10" aria-hidden="true"><path d="M1.5 1.5l7 7M8.5 1.5l-7 7" /></svg>
    </button>
  </div>
</header>

<style>
  .titlebar {
    display: flex;
    align-items: stretch;
    justify-content: space-between;
    height: 34px;
    border-bottom: 1px solid var(--border);
    background: var(--rail);
    user-select: none;
  }
  .title {
    display: flex;
    align-items: center;
    gap: var(--s-3);
    padding-left: var(--s-4);
    font-family: var(--font-mono);
    font-size: 11px;
    letter-spacing: 0.1em;
  }
  .title b {
    color: var(--accent);
    text-shadow: var(--glow-text);
    pointer-events: none;
  }
  .title i {
    color: var(--text-3);
    font-style: normal;
    text-transform: uppercase;
  }
  .controls {
    display: flex;
  }
  button {
    display: grid;
    place-items: center;
    width: 46px;
    border: 0;
    border-left: 1px solid transparent;
    background: transparent;
    color: var(--text-2);
    cursor: pointer;
  }
  button svg {
    width: 10px;
    height: 10px;
    fill: none;
    stroke: currentColor;
    stroke-width: 1.2;
  }
  button:hover {
    background: var(--surface-2);
    color: var(--accent);
  }
  button.close:hover {
    background: var(--danger);
    color: #fff;
  }
</style>
