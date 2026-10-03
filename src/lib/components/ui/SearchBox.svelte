<script lang="ts">
  import Icon from "./Icon.svelte";

  let {
    value = $bindable(),
    placeholder,
    label,
  }: { value: string; placeholder: string; label: string } = $props();
</script>

<label class="search">
  <Icon name="search" size={15} />
  <input type="search" aria-label={label} {placeholder} bind:value />
</label>

<style>
  .search {
    display: flex;
    align-items: center;
    gap: var(--s-2);
    width: 300px;
    padding: 7px var(--s-3);
    border: 1px solid var(--border);
    background: var(--surface);
    color: var(--text-3);
  }
  .search:focus-within {
    border-color: var(--accent);
    box-shadow: var(--glow);
  }
  input {
    flex: 1;
    min-width: 0;
    border: 0;
    outline: 0;
    background: transparent;
    font-family: var(--font-mono);
    font-size: var(--fs-small);
  }
</style>
