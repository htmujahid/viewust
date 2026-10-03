<script lang="ts">
  import { onDestroy, onMount } from "svelte";

  import { goto } from "$app/navigation";
  import { page } from "$app/state";

  import LiveToggle from "$lib/components/ui/LiveToggle.svelte";
  import Page from "$lib/components/ui/Page.svelte";
  import TabNav from "$lib/components/ui/TabNav.svelte";
  import { monitor } from "$lib/features/monitor/store.svelte";
  import { hasPower, hasThermal } from "$lib/features/monitor/thermal";
  import "$lib/features/monitor/monitor.css";

  let { children } = $props();

  onMount(monitor.start);
  onDestroy(monitor.stop);

  const tabs = $derived([
    { href: "/monitor", label: "Overview" },
    { href: "/monitor/cpu", label: "Processor" },
    { href: "/monitor/memory", label: "Memory" },
    { href: "/monitor/storage", label: "Storage" },
    ...(monitor.latest?.gpus.length ? [{ href: "/monitor/gpu", label: "Graphics" }] : []),
    ...(hasThermal(monitor.latest) ? [{ href: "/monitor/thermal", label: "Thermal" }] : []),
    ...(hasPower(monitor.latest) ? [{ href: "/monitor/power", label: "Power" }] : []),
    { href: "/monitor/network", label: "Network" },
  ]);

  function onkeydown(e: KeyboardEvent) {
    if (e.key !== "Escape" || (e.target as HTMLElement)?.closest("svg")) return;
    goto(page.url.pathname === "/monitor" ? "/" : "/monitor");
  }
</script>

<svelte:window {onkeydown} />

<Page>
  <div class="bar">
    <TabNav {tabs} current={page.url.pathname} label="Monitor sections" />
    <LiveToggle bind:live={monitor.live} />
  </div>

  {#if monitor.error && !monitor.latest}
    <p class="error">Couldn't read the system: {monitor.error}</p>
  {:else if !monitor.latest}
    <div class="skeleton" style="height: 320px"></div>
  {:else}
    <div class="content" class:stale={!monitor.live}>
      {@render children()}
    </div>
  {/if}
</Page>

<style>
  .bar {
    display: flex;
    flex-wrap: wrap;
    align-items: flex-end;
    justify-content: space-between;
    gap: var(--s-3);
    margin-bottom: var(--s-4);
  }
  .bar :global(nav) {
    padding: 0;
  }
  .bar :global(nav a) {
    margin-bottom: -1px;
  }
  .bar :global(.btn) {
    margin-bottom: 2px;
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
