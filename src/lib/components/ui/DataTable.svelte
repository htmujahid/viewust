<script lang="ts" module>
  export interface Column {
    key: string;
    label: string;
    right?: boolean;
    hint?: string;
    sortable?: boolean;
  }
</script>

<script lang="ts" generics="T">
  import type { Snippet } from "svelte";

  let {
    rows,
    columns,
    rowKey,
    selectedKey = null,
    sortKey,
    sortDesc,
    emptyText,
    onsort,
    onselect,
    oncontext,
    cell,
  }: {
    rows: readonly T[];
    columns: readonly Column[];
    rowKey: (row: T) => string;
    selectedKey?: string | null;
    sortKey: string;
    sortDesc: boolean;
    emptyText: string;
    onsort: (key: string) => void;
    onselect: (key: string) => void;
    /** Called on right-click; the table doesn't know what the menu holds. */
    oncontext?: (row: T, event: MouseEvent) => void;
    cell: Snippet<[T, Column]>;
  } = $props();

  const ariaSort = (key: string) =>
    sortKey === key ? (sortDesc ? "descending" : "ascending") : "none";
</script>

<div class="card wrap grow">
  <div class="scroll">
    <table>
      <thead>
        <tr>
          {#each columns as c (c.key)}
            <th class:right={c.right} aria-sort={ariaSort(c.key)} title={c.hint}>
              {#if c.sortable === false}
                <span class="plain">{c.label}</span>
              {:else}
                <button onclick={() => onsort(c.key)}>
                  {c.label}
                  <span class="arrow" class:active={sortKey === c.key}>
                    {sortKey === c.key ? (sortDesc ? "↓" : "↑") : "↕"}
                  </span>
                </button>
              {/if}
            </th>
          {/each}
        </tr>
      </thead>
      <tbody>
        {#each rows as row (rowKey(row))}
          <tr
            class:selected={selectedKey === rowKey(row)}
            onclick={() => onselect(rowKey(row))}
            oncontextmenu={oncontext && ((e) => oncontext(row, e))}
          >
            {#each columns as c (c.key)}
              <td class:right={c.right}>{@render cell(row, c)}</td>
            {/each}
          </tr>
        {:else}
          <tr><td colspan={columns.length} class="empty">{emptyText}</td></tr>
        {/each}
      </tbody>
    </table>
  </div>
</div>

<style>
  .wrap {
    display: flex;
    flex-direction: column;
    min-height: 160px;
  }
  .scroll {
    flex: 1;
    min-height: 0;
    overflow: auto;
  }
  table {
    width: 100%;
    border-collapse: collapse;
  }
  thead th {
    position: sticky;
    top: 0;
    z-index: 1;
    padding: 0;
    background: var(--surface);
    border-bottom: 1px solid var(--accent);
    text-align: left;
  }
  th button,
  .plain {
    display: block;
    width: 100%;
    padding: 12px var(--s-4);
    border: 0;
    background: transparent;
    color: var(--text-2);
    font-family: var(--font-mono);
    font-size: 11px;
    font-weight: 600;
    letter-spacing: 0.08em;
    text-align: inherit;
    text-transform: uppercase;
    white-space: nowrap;
  }
  th button {
    cursor: pointer;
  }
  th.right,
  td.right {
    text-align: right;
  }
  th button:hover {
    color: var(--text);
  }
  .arrow {
    margin-left: 2px;
    color: var(--text-3);
    opacity: 0.5;
  }
  .arrow.active {
    color: var(--accent);
    opacity: 1;
  }
  tbody tr {
    cursor: pointer;
  }
  tbody tr:hover {
    background: var(--surface-2);
  }
  tbody tr.selected {
    background: var(--accent-soft);
    box-shadow: inset 3px 0 0 var(--accent);
  }
  td {
    padding: 10px var(--s-4);
    border-bottom: 1px solid var(--border);
    font-family: var(--font-mono);
    font-size: var(--fs-small);
    white-space: nowrap;
  }
  tbody tr:last-child td {
    border-bottom: 0;
  }
  .empty {
    padding: var(--s-6);
    color: var(--text-3);
    text-align: center;
  }
</style>
