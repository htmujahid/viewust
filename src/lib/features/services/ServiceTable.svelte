<script lang="ts">
  import type { ServiceRow } from "$lib/api/types";
  import Badge from "$lib/components/ui/Badge.svelte";
  import Meter from "$lib/components/ui/Meter.svelte";
  import { formatBytes } from "$lib/utils/format";

  import { bootOf, statusOf } from "./status";
  import type { SortKey } from "./store.svelte";

  let {
    rows,
    sortKey,
    sortDesc,
    selectedUnit,
    emptyText,
    onsort,
    onselect,
  }: {
    rows: readonly ServiceRow[];
    sortKey: SortKey;
    sortDesc: boolean;
    selectedUnit: string | null;
    emptyText: string;
    onsort: (key: SortKey) => void;
    onselect: (unit: string) => void;
  } = $props();

  const columns: { key: SortKey; label: string; right?: boolean; hint?: string }[] = [
    { key: "name", label: "Service" },
    { key: "state", label: "Status" },
    { key: "enabled", label: "At boot", hint: "Whether the service starts when the computer does" },
    { key: "main_pid", label: "PID", right: true },
    {
      key: "memory",
      label: "Memory",
      hint: "Memory held by the service and everything it started",
    },
  ];

  const biggest = $derived(Math.max(1, ...rows.map((r) => r.memory ?? 0)));
  const ariaSort = (key: SortKey) =>
    sortKey === key ? (sortDesc ? "descending" : "ascending") : "none";
</script>

<div class="card wrap grow">
  <table>
    <thead>
      <tr>
        {#each columns as c}
          <th class:right={c.right} aria-sort={ariaSort(c.key)} title={c.hint}>
            <button onclick={() => onsort(c.key)}>
              {c.label}
              <span class="arrow" class:active={sortKey === c.key}>
                {sortKey === c.key ? (sortDesc ? "↓" : "↑") : "↕"}
              </span>
            </button>
          </th>
        {/each}
      </tr>
    </thead>
    <tbody>
      {#each rows as r (r.unit)}
        {@const status = statusOf(r)}
        {@const boot = bootOf(r.enabled)}
        <tr class:selected={selectedUnit === r.unit} onclick={() => onselect(r.unit)}>
          <td class="name">
            <span class="truncate" title={r.unit}>{r.name}</span>
            {#if r.description}<small class="truncate muted" title={r.description}
                >{r.description}</small
              >{/if}
          </td>
          <td><Badge tone={status.tone}>{status.label}</Badge></td>
          <td><Badge tone={boot.tone}>{boot.label}</Badge></td>
          <td class="right tabular muted">{r.main_pid ?? "—"}</td>
          <td class="mem">
            {#if r.memory !== null}
              <span class="tabular">{formatBytes(r.memory)}</span>
              <Meter value={(r.memory / biggest) * 100} tone="accent" height={4} />
            {:else}
              <span class="muted">—</span>
            {/if}
          </td>
        </tr>
      {:else}
        <tr><td colspan="5" class="empty">{emptyText}</td></tr>
      {/each}
    </tbody>
  </table>
</div>

<style>
  .wrap {
    overflow: auto;
    min-height: 160px;
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
  th button {
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
    cursor: pointer;
    white-space: nowrap;
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
  td.name {
    max-width: 420px;
  }
  td.name span {
    display: block;
    font-weight: 600;
  }
  td.name small {
    display: block;
    font-family: var(--font-ui);
  }
  td.mem {
    min-width: 130px;
  }
  td.mem :global(.meter) {
    margin-top: 3px;
  }
  .muted {
    color: var(--text-2);
  }
  .empty {
    padding: var(--s-6);
    color: var(--text-3);
    text-align: center;
  }
</style>
