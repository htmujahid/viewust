<script lang="ts">
  import "../app.css";

  import AppShell from "$lib/components/ui/AppShell.svelte";
  import { onMount } from "svelte";

  import { backendAvailable } from "$lib/api/client";
  import NoBackend from "$lib/components/NoBackend.svelte";
  import { theme } from "$lib/stores/theme.svelte";

  let { children } = $props();

  onMount(() => theme.init());
</script>

{#if backendAvailable()}
  <AppShell>{@render children()}</AppShell>
{:else}
  <NoBackend />
{/if}
