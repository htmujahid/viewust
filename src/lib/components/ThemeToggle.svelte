<script lang="ts">
  import { theme, type ThemeMode } from "$lib/theme.svelte";
  import Icon, { type IconName } from "./Icon.svelte";

  const options: { mode: ThemeMode; label: string; icon: IconName }[] = [
    { mode: "light", label: "Light", icon: "sun" },
    { mode: "system", label: "System", icon: "system" },
    { mode: "dark", label: "Dark", icon: "moon" },
  ];
</script>

<div class="toggle" role="radiogroup" aria-label="Theme">
  {#each options as o}
    <button
      role="radio"
      aria-checked={theme.mode === o.mode}
      aria-label={o.label}
      title={o.label}
      onclick={() => theme.set(o.mode)}
    >
      <Icon name={o.icon} size={15} />
    </button>
  {/each}
</div>

<style>
  .toggle {
    display: flex;
    gap: 2px;
    padding: 3px;
    border-radius: var(--radius-sm);
    background: var(--surface-2);
  }
  button {
    display: grid;
    place-items: center;
    width: 28px;
    height: 26px;
    border: 0;
    border-radius: 8px;
    background: transparent;
    color: var(--text-3);
    cursor: pointer;
  }
  button:hover {
    color: var(--text);
  }
  button[aria-checked="true"] {
    background: var(--surface);
    color: var(--accent);
    box-shadow: var(--shadow), 0 0 0 1px var(--border);
  }
</style>
