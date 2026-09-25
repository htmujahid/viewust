<script lang="ts">
  import { onDestroy, onMount } from "svelte";
  import { goto } from "$app/navigation";
  import { page } from "$app/state";
  import Icon from "$lib/components/Icon.svelte";
  import ThemeToggle from "$lib/components/ThemeToggle.svelte";
  import { monitor } from "$lib/monitor.svelte";

  let { children } = $props();

  // Sampling lives here, in the section's layout, so the history keeps building
  // while you move between the pages below.
  onMount(() => monitor.start());
  onDestroy(() => monitor.stop());

  const gpus = $derived(monitor.latest?.gpus.length ?? 0);
  const tabs = $derived([
    { href: "/monitor", label: "Overview" },
    { href: "/monitor/cpu", label: "Processor" },
    { href: "/monitor/memory", label: "Memory" },
    { href: "/monitor/storage", label: "Storage" },
    ...(gpus ? [{ href: "/monitor/gpu", label: "Graphics" }] : []),
    { href: "/monitor/network", label: "Network" },
  ]);

  function onkeydown(e: KeyboardEvent) {
    if (e.key !== "Escape" || (e.target as HTMLElement)?.closest("svg")) return;
    goto(page.url.pathname === "/monitor" ? "/" : "/monitor");
  }
</script>

<svelte:window {onkeydown} />

<div class="page">
  <header class="top">
    <button class="btn" onclick={() => goto("/")}>
      <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M19 12H5M11 6l-6 6 6 6" /></svg>
      Devices
    </button>
    <div class="heading">
      <h1>Live monitor</h1>
      <p>Updated every second</p>
    </div>
    <div class="spacer"></div>
    <button class="btn" onclick={() => goto("/processes")}>
      <Icon name="activity" size={16} />
      Processes
    </button>
    <button class="btn" class:on={monitor.live} onclick={() => (monitor.live = !monitor.live)} aria-pressed={monitor.live}>
      <span class="dot" class:pulse={monitor.live}></span>
      {monitor.live ? "Live" : "Paused"}
    </button>
    <ThemeToggle />
  </header>

  <nav class="tabs" aria-label="Monitor sections">
    {#each tabs as t}
      <a href={t.href} aria-current={page.url.pathname === t.href ? "page" : undefined}>{t.label}</a>
    {/each}
  </nav>

  {#if monitor.error && !monitor.latest}
    <p class="error">Couldn't read the system: {monitor.error}</p>
  {:else if !monitor.latest}
    <div class="skeleton" style="height: 320px"></div>
  {:else}
    <div class="content" class:stale={!monitor.live}>
      {@render children()}
    </div>
  {/if}
</div>

<style>
  .page {
    height: 100vh;
    overflow-y: auto;
    padding: var(--s-4) var(--s-5) var(--s-6);
  }
  .page > :global(*) {
    max-width: 1180px;
    margin-left: auto;
    margin-right: auto;
  }
  .top {
    display: flex;
    align-items: center;
    gap: var(--s-3);
    margin-bottom: var(--s-4);
  }
  .heading h1 {
    font-size: 20px;
    font-weight: 650;
  }
  .heading p {
    color: var(--text-2);
    font-size: var(--fs-small);
  }
  .spacer {
    flex: 1;
  }
  .dot {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    background: var(--text-3);
  }
  .btn.on .dot {
    background: var(--ok);
  }
  .dot.pulse {
    animation: pulse 1.6s ease-in-out infinite;
  }
  @keyframes pulse {
    50% {
      opacity: 0.35;
    }
  }

  .tabs {
    display: flex;
    flex-wrap: wrap;
    gap: var(--s-1);
    margin-bottom: var(--s-4);
    padding: var(--s-1);
    border-radius: 12px;
    background: var(--surface-2);
    width: fit-content;
  }
  .tabs a {
    padding: 6px var(--s-4);
    border-radius: 9px;
    color: var(--text-2);
    font-weight: 500;
    text-decoration: none;
  }
  .tabs a:hover {
    color: var(--text);
  }
  .tabs a[aria-current="page"] {
    background: var(--surface);
    color: var(--text);
    box-shadow: var(--shadow);
  }
  .content {
    transition: opacity 0.2s;
  }
  .content.stale {
    opacity: 0.75;
  }
  .error {
    padding: var(--s-6) 0;
    color: var(--danger);
    text-align: center;
  }
</style>
