<script lang="ts">
  import { onDestroy, onMount } from "svelte";

  import { goto } from "$app/navigation";
  import { page } from "$app/state";

  import LiveToggle from "$lib/components/ui/LiveToggle.svelte";
  import NavLinks from "$lib/components/ui/NavLinks.svelte";
  import Page from "$lib/components/ui/Page.svelte";
  import PageHeader from "$lib/components/ui/PageHeader.svelte";
  import TabNav from "$lib/components/ui/TabNav.svelte";
  import { monitor } from "$lib/features/monitor/store.svelte";
  import "$lib/features/monitor/monitor.css";

  let { children } = $props();

  // Sampling lives here, in the section's layout, so the history keeps building
  // while you move between the pages below.
  onMount(monitor.start);
  onDestroy(monitor.stop);

  const tabs = $derived([
    { href: "/monitor", label: "Overview" },
    { href: "/monitor/cpu", label: "Processor" },
    { href: "/monitor/memory", label: "Memory" },
    { href: "/monitor/storage", label: "Storage" },
    ...(monitor.latest?.gpus.length ? [{ href: "/monitor/gpu", label: "Graphics" }] : []),
    { href: "/monitor/network", label: "Network" },
  ]);

  function onkeydown(e: KeyboardEvent) {
    if (e.key !== "Escape" || (e.target as HTMLElement)?.closest("svg")) return;
    goto(page.url.pathname === "/monitor" ? "/" : "/monitor");
  }
</script>

<svelte:window {onkeydown} />

<Page>
  <PageHeader back="/" backLabel="Devices" title="Live monitor" subtitle="Updated every second">
    {#snippet actions()}
      <NavLinks hide={["monitor"]} />
      <LiveToggle bind:live={monitor.live} />
    {/snippet}
  </PageHeader>

  <div class="tabs"><TabNav {tabs} current={page.url.pathname} label="Monitor sections" /></div>

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
  .tabs {
    margin-bottom: var(--s-4);
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
