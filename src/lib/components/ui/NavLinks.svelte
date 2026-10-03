<script lang="ts">
  import Icon from "./Icon.svelte";

  let { hide = [] }: { hide?: ("processes" | "monitor")[] } = $props();
</script>

{#if !hide.includes("processes")}
  <a class="btn" href="/processes"><Icon name="activity" size={16} /> Processes</a>
{/if}
{#if !hide.includes("monitor")}
  <a class="btn" href="/monitor"><Icon name="chart" size={16} /> Live monitor</a>
{/if}
