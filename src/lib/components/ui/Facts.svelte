<script lang="ts">
  /** A row of small label/value pairs. Items with a null value are skipped. */
  let { items }: { items: { label: string; value: string | null }[] } = $props();
</script>

<dl class="facts">
  {#each items.filter((i) => i.value !== null) as i}
    <div>
      <dt>{i.label}</dt>
      <dd class="tabular">{i.value}</dd>
    </div>
  {/each}
</dl>

<style>
  .facts {
    display: flex;
    flex-wrap: wrap;
    gap: var(--s-2) var(--s-5);
    margin: 0;
  }
  dt {
    color: var(--text-3);
    font-size: var(--fs-label);
  }
  dd {
    margin: 0;
    font-weight: 600;
  }
</style>
