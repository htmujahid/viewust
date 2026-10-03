<script lang="ts">
  import type { Detail } from "$lib/api/types";

  /**
   * Label/value rows. A value that is long, or spans several lines (a hex dump,
   * a command line), drops under its label instead of squeezing beside it.
   */
  let {
    rows,
    stackAt = 56,
  }: { rows: readonly (Pick<Detail, "label" | "value"> & { hint?: string })[]; stackAt?: number } =
    $props();
</script>

<dl>
  {#each rows as row}
    <div class="row" class:stacked={row.value.length > stackAt || row.value.includes("\n")}>
      <dt title={row.hint}>{row.label}</dt>
      {#if row.value.includes("\n")}
        <dd><pre>{row.value}</pre></dd>
      {:else}
        <dd>{row.value}</dd>
      {/if}
    </div>
  {/each}
</dl>

<style>
  dl {
    margin: 0;
  }
  .row {
    display: flex;
    justify-content: space-between;
    gap: var(--s-4);
    padding: 7px 0;
    border-bottom: 1px solid var(--border);
  }
  .row:last-child {
    border-bottom: 0;
  }
  .row.stacked {
    flex-direction: column;
    gap: 2px;
  }
  dt {
    flex: none;
    color: var(--text-2);
    font-family: var(--font-mono);
    font-size: var(--fs-small);
  }
  dd {
    margin: 0;
    min-width: 0;
    font-family: var(--font-mono);
    font-size: var(--fs-small);
    font-variant-numeric: tabular-nums;
    font-weight: 500;
    text-align: right;
    overflow-wrap: anywhere;
    user-select: text;
  }
  .stacked dd {
    text-align: left;
  }
  pre {
    margin: 0;
    padding: var(--s-3);
    border-radius: 0;
    background: var(--bg);
    border: 1px solid var(--border);
    border-left: 2px solid var(--accent);
    color: var(--ok);
    font: 12px/1.5 var(--font-mono);
    font-weight: 400;
    overflow-x: auto;
  }
</style>
