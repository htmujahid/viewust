<script lang="ts">
  import { theme } from "$lib/stores/theme.svelte";

  import Icon from "./Icon.svelte";

  const dark = $derived(theme.effective === "dark");
  const label = $derived(dark ? "Switch to the light theme" : "Switch to the dark theme");
</script>

<button class="toggle" aria-label={label} title={label} onclick={() => theme.toggle()}>
  <Icon name={dark ? "sun" : "moon"} size={16} />
</button>

<style>
  .toggle {
    display: grid;
    place-items: center;
    width: 34px;
    height: 32px;
    border: 1px solid var(--border);
    background: var(--surface-2);
    color: var(--text-2);
    cursor: pointer;
  }
  .toggle:hover {
    border-color: var(--accent);
    color: var(--accent);
    box-shadow: var(--glow);
  }
</style>
