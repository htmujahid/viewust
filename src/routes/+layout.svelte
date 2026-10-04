<script lang="ts">
  import "../app.css";

  import AppShell from "$lib/components/ui/AppShell.svelte";
  import ConfirmDialog from "$lib/components/ui/ConfirmDialog.svelte";
  import ContextMenu from "$lib/components/ui/ContextMenu.svelte";
  import Toasts from "$lib/components/ui/Toasts.svelte";
  import { onMount } from "svelte";

  import { backendAvailable } from "$lib/api/client";
  import NoBackend from "$lib/components/NoBackend.svelte";
  import { theme } from "$lib/stores/theme.svelte";

  let { children } = $props();

  onMount(() => theme.init());

  // The browser's own menu (Reload, Inspect…) means nothing here; keep it only where text is edited.
  function oncontextmenu(e: MouseEvent) {
    if (!(e.target instanceof HTMLElement && e.target.closest("input, textarea"))) {
      e.preventDefault();
    }
  }
</script>

<svelte:window {oncontextmenu} />

{#if backendAvailable()}
  <AppShell>{@render children()}</AppShell>
  <ContextMenu />
  <ConfirmDialog />
  <Toasts />
{:else}
  <NoBackend />
{/if}
