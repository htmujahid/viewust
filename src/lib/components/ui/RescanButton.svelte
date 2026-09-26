<script lang="ts">
  import Icon from "./Icon.svelte";

  let {
    loading,
    onclick,
    label = false,
  }: {
    loading: boolean;
    onclick: () => void;
    /** Show "Rescan" / "Scanning" next to the icon. */
    label?: boolean;
  } = $props();
</script>

<button class="btn" {onclick} disabled={loading} aria-label="Rescan">
  <span class="spin-wrap" class:spin={loading}><Icon name="refresh" size={16} /></span>
  {#if label}{loading ? "Scanning" : "Rescan"}{/if}
</button>

<style>
  .spin-wrap {
    display: inline-flex;
  }
  .spin {
    animation: spin 0.9s linear infinite;
  }
</style>
