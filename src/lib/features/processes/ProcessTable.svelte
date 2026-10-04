<script lang="ts">
  import type { ProcessRow } from "$lib/api/types";
  import Meter from "$lib/components/ui/Meter.svelte";
  import { formatBytes } from "$lib/utils/format";

  import type { SortKey } from "./store.svelte";

  let {
    rows,
    memoryTotal,
    sortKey,
    sortDesc,
    selectedPid,
    emptyText,
    onsort,
    onselect,
    oncontext,
  }: {
    rows: readonly ProcessRow[];
    memoryTotal: number;
    sortKey: SortKey;
    sortDesc: boolean;
    selectedPid: number | null;
    emptyText: string;
    onsort: (key: SortKey) => void;
    onselect: (pid: number) => void;
    oncontext: (row: ProcessRow, event: MouseEvent) => void;
  } = $props();

  const columns: { key: SortKey; label: string; right?: boolean; hint?: string }[] = [
    { key: "name", label: "Name" },
    { key: "pid", label: "PID", right: true },
    { key: "user", label: "Owner" },
    {
      key: "cpu",
      label: "CPU",
      right: true,
      hint: "Percent of one core; multi-threaded programs can exceed 100%",
    },
    { key: "memory", label: "In RAM", hint: "Memory the process occupies right now" },
    {
      key: "virtual_memory",
      label: "Asked for",
      right: true,
      hint: "Address space the process has requested from the system",
    },
    { key: "threads", label: "Threads", right: true },
  ];

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
      {#each rows as p (p.pid)}
        <tr
          class:selected={selectedPid === p.pid}
          onclick={() => onselect(p.pid)}
          oncontextmenu={(e) => oncontext(p, e)}
        >
          <td class="name"><span class="truncate" title={p.name}>{p.name}</span></td>
          <td class="right tabular muted">{p.pid}</td>
          <td class="muted">{p.user}</td>
          <td class="right tabular" class:hot={p.cpu >= 50}>{p.cpu.toFixed(1)}%</td>
          <td class="mem">
            <span class="tabular">{formatBytes(p.memory)}</span>
            <Meter value={(p.memory / memoryTotal) * 100 * 8} tone="accent" height={4} />
          </td>
          <td class="right tabular muted">{formatBytes(p.virtual_memory)}</td>
          <td class="right tabular muted">{p.threads}</td>
        </tr>
      {:else}
        <tr><td colspan="7" class="empty">{emptyText}</td></tr>
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
    padding: 11px var(--s-4);
    border-bottom: 1px solid var(--border);
    font-family: var(--font-mono);
    font-size: var(--fs-small);
    white-space: nowrap;
  }
  tbody tr:last-child td {
    border-bottom: 0;
  }
  td.name {
    max-width: 260px;
    font-weight: 600;
  }
  td.name .truncate {
    display: block;
  }
  td.mem {
    min-width: 130px;
  }
  td.mem :global(.meter) {
    margin-top: 3px;
  }
  td.hot {
    color: var(--danger);
    font-weight: 600;
    text-shadow: 0 0 10px color-mix(in srgb, var(--danger) 55%, transparent);
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
