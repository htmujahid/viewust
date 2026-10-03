<script lang="ts" generics="K extends string">
  let {
    options,
    value = $bindable(),
    label,
  }: {
    options: readonly { key: K; label: string; count?: number }[];
    value: K;
    label: string;
  } = $props();
</script>

<div class="chips" role="group" aria-label={label}>
  {#each options as o (o.key)}
    <button class="chip" aria-pressed={value === o.key} onclick={() => (value = o.key)}>
      {o.label}
      {#if o.count !== undefined}<span class="tabular">{o.count}</span>{/if}
    </button>
  {/each}
</div>

<style>
  .chips {
    display: flex;
    flex-wrap: wrap;
    gap: var(--s-2);
  }
  .chip {
    display: inline-flex;
    align-items: center;
    gap: var(--s-2);
    padding: 5px var(--s-3);
    border: 1px solid var(--border);
    background: var(--surface);
    color: var(--text-2);
    font-family: var(--font-mono);
    font-size: 11px;
    letter-spacing: 0.05em;
    text-transform: uppercase;
    cursor: pointer;
  }
  .chip span {
    color: var(--text-3);
  }
  .chip:hover {
    border-color: var(--border-strong);
    color: var(--text);
  }
  .chip[aria-pressed="true"] {
    border-color: var(--accent);
    background: var(--accent-soft);
    color: var(--accent);
    box-shadow: var(--glow);
  }
  .chip[aria-pressed="true"] span {
    color: var(--accent);
  }
</style>
