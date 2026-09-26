<script lang="ts">
  import "../app.css";

  import { onMount } from "svelte";

  import { backendAvailable } from "$lib/api/client";
  import NoBackend from "$lib/components/NoBackend.svelte";
  import { theme } from "$lib/stores/theme.svelte";

  let { children } = $props();

  onMount(() => theme.init());
</script>

{#if backendAvailable()}
  {@render children()}
{:else}
  <NoBackend />
{/if}
