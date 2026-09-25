<script lang="ts">
  import { onDestroy, onMount } from "svelte";
  import { goto } from "$app/navigation";
  import Icon from "$lib/components/Icon.svelte";
  import Meter from "$lib/components/Meter.svelte";
  import ProcessPanel from "$lib/components/ProcessPanel.svelte";
  import ThemeToggle from "$lib/components/ThemeToggle.svelte";
  import { processes, type SortKey } from "$lib/processes.svelte";
  import { formatBytes, formatDuration } from "$lib/system";

  const o = $derived(processes.snapshot?.overview ?? null);
  const rows = $derived(processes.rows);
  const selectedRow = $derived(processes.snapshot?.processes.find((p) => p.pid === processes.selectedPid));

  const memPct = $derived(o ? (o.memory_used / o.memory_total) * 100 : 0);
  const swapPct = $derived(o && o.swap_total ? (o.swap_used / o.swap_total) * 100 : 0);
  // Machine-wide CPU is already 0–100 across all cores.
  const cpuPct = $derived(o?.cpu ?? 0);

  const columns: { key: SortKey; label: string; align?: "right"; hint?: string }[] = [
    { key: "name", label: "Name" },
    { key: "pid", label: "PID", align: "right" },
    { key: "user", label: "Owner" },
    { key: "cpu", label: "CPU", align: "right", hint: "Percent of one core; multi-threaded programs can exceed 100%" },
    { key: "memory", label: "In RAM", hint: "Memory the process occupies right now" },
    { key: "virtual_memory", label: "Asked for", align: "right", hint: "Address space the process has requested from the system" },
    { key: "threads", label: "Threads", align: "right" },
  ];

  function onkeydown(e: KeyboardEvent) {
    if (e.key !== "Escape") return;
    if (processes.selectedPid !== null) processes.select(null);
    else goto("/");
  }

  onMount(() => processes.start());
  onDestroy(() => processes.stop());
</script>

<svelte:window {onkeydown} />

<div class="layout">
  <div class="page">
    <header class="top">
      <button class="btn" onclick={() => goto("/")}>
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M19 12H5M11 6l-6 6 6 6" /></svg>
        Devices
      </button>
      <div class="heading">
        <h1>Processes</h1>
        <p>{o ? `${o.processes} running · ${o.threads} threads` : "Reading…"}</p>
      </div>
      <div class="spacer"></div>
      <label class="search">
        <Icon name="search" size={15} />
        <input type="search" placeholder="Search name, owner or PID" bind:value={processes.search} />
      </label>
      <button class="btn" onclick={() => goto("/monitor")}>
        <Icon name="chart" size={16} />
        Live monitor
      </button>
      <button class="btn" class:on={processes.live} onclick={() => (processes.live = !processes.live)} aria-pressed={processes.live}>
        <span class="dot" class:pulse={processes.live}></span>
        {processes.live ? "Live" : "Paused"}
      </button>
      <ThemeToggle />
    </header>

    {#if processes.error && !o}
      <p class="error">Couldn't read processes: {processes.error}</p>
    {:else if !o}
      <div class="skeleton" style="height: 96px; margin-bottom: 16px"></div>
      <div class="skeleton" style="height: 420px"></div>
    {:else}
      <section class="tiles">
        <div class="card tile">
          <p class="label">Memory</p>
          <p class="value tabular">{formatBytes(o.memory_used)} <small>of {formatBytes(o.memory_total)}</small></p>
          <Meter value={memPct} />
        </div>
        <div class="card tile">
          <p class="label">CPU</p>
          <p class="value tabular">{cpuPct.toFixed(0)}% <small>across {o.cpu_count} threads</small></p>
          <Meter value={cpuPct} />
        </div>
        <div class="card tile">
          <p class="label">Swap</p>
          <p class="value tabular">
            {o.swap_total ? formatBytes(o.swap_used) : "None"}
            {#if o.swap_total}<small>of {formatBytes(o.swap_total)}</small>{/if}
          </p>
          <Meter value={swapPct} />
        </div>
        <div class="card tile">
          <p class="label">Load average</p>
          <p class="value tabular">{o.load[0].toFixed(2)} <small>{o.load[1].toFixed(2)} · {o.load[2].toFixed(2)}</small></p>
          <p class="sub">1, 5 and 15 minutes · up {formatDuration(o.uptime)}</p>
        </div>
      </section>

      <div class="toolbar">
        <p class="muted">
          {rows.length}
          {rows.length === 1 ? "process" : "processes"}{processes.search ? " match" : ""}
        </p>
        <label class="check">
          <input type="checkbox" bind:checked={processes.showKernel} />
          Show kernel threads
        </label>
      </div>

      <div class="card table-wrap">
        <table>
          <thead>
            <tr>
              {#each columns as c}
                <th
                  class:right={c.align === "right"}
                  aria-sort={processes.sortKey === c.key ? (processes.sortDesc ? "descending" : "ascending") : "none"}
                  title={c.hint}
                >
                  <button onclick={() => processes.sortBy(c.key)}>
                    {c.label}
                    <span class="arrow" class:active={processes.sortKey === c.key}>
                      {processes.sortKey === c.key ? (processes.sortDesc ? "↓" : "↑") : "↕"}
                    </span>
                  </button>
                </th>
              {/each}
            </tr>
          </thead>
          <tbody>
            {#each rows as p (p.pid)}
              <tr class:selected={processes.selectedPid === p.pid} onclick={() => processes.select(p.pid)}>
                <td class="name"><span class="truncate" title={p.name}>{p.name}</span></td>
                <td class="right tabular muted">{p.pid}</td>
                <td class="muted">{p.user}</td>
                <td class="right tabular" class:hot={p.cpu >= 50}>{p.cpu.toFixed(1)}%</td>
                <td class="mem">
                  <span class="tabular">{formatBytes(p.memory)}</span>
                  <Meter value={(p.memory / o.memory_total) * 100 * 8} tone="accent" height={4} />
                </td>
                <td class="right tabular muted">{formatBytes(p.virtual_memory)}</td>
                <td class="right tabular muted">{p.threads}</td>
              </tr>
            {:else}
              <tr><td colspan="7" class="empty">No processes match “{processes.search}”.</td></tr>
            {/each}
          </tbody>
        </table>
      </div>
    {/if}
  </div>

  {#if processes.selectedPid !== null}
    <ProcessPanel
      row={selectedRow}
      detail={processes.detail}
      onclose={() => processes.select(null)}
      onselect={(pid) => processes.select(pid)}
    />
  {/if}
</div>

<style>
  .layout {
    display: flex;
    height: 100vh;
  }
  .page {
    flex: 1;
    min-width: 0;
    overflow-y: auto;
    padding: var(--s-4) var(--s-5) var(--s-6);
  }

  .top {
    display: flex;
    align-items: center;
    gap: var(--s-3);
    margin-bottom: var(--s-4);
  }
  .heading h1 {
    font-size: 20px;
    font-weight: 650;
  }
  .heading p {
    color: var(--text-2);
    font-size: var(--fs-small);
  }
  .spacer {
    flex: 1;
  }
  .search {
    display: flex;
    align-items: center;
    gap: var(--s-2);
    width: 260px;
    padding: 7px var(--s-3);
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
    background: var(--surface);
    color: var(--text-3);
  }
  .search:focus-within {
    border-color: var(--accent);
  }
  .search input {
    flex: 1;
    min-width: 0;
    border: 0;
    outline: 0;
    background: transparent;
  }
  .dot {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    background: var(--text-3);
  }
  .btn.on .dot {
    background: var(--ok);
  }
  .dot.pulse {
    animation: pulse 1.6s ease-in-out infinite;
  }
  @keyframes pulse {
    50% {
      opacity: 0.35;
    }
  }

  .tiles {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(210px, 1fr));
    gap: var(--s-4);
    margin-bottom: var(--s-5);
  }
  .tile {
    display: flex;
    flex-direction: column;
    gap: var(--s-2);
    padding: var(--s-4);
  }
  .label {
    color: var(--text-3);
    font-size: var(--fs-label);
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.06em;
  }
  .value {
    font-size: 22px;
    font-weight: 650;
    letter-spacing: -0.01em;
  }
  .value small,
  .sub {
    color: var(--text-2);
    font-size: var(--fs-small);
    font-weight: 400;
  }

  .toolbar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: var(--s-2);
  }
  .muted {
    color: var(--text-2);
  }
  .check {
    display: inline-flex;
    align-items: center;
    gap: var(--s-2);
    color: var(--text-2);
    cursor: pointer;
  }

  .table-wrap {
    overflow: auto;
    max-height: calc(100vh - 292px);
    min-height: 240px;
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
    border-bottom: 1px solid var(--border);
    text-align: left;
  }
  th button {
    width: 100%;
    padding: 9px var(--s-3);
    border: 0;
    background: transparent;
    color: var(--text-2);
    font-size: var(--fs-small);
    font-weight: 600;
    text-align: inherit;
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
  }
  td {
    padding: 7px var(--s-3);
    border-bottom: 1px solid var(--border);
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
  }
  .empty {
    padding: var(--s-6);
    color: var(--text-3);
    text-align: center;
  }
  .error {
    padding: var(--s-6) 0;
    color: var(--danger);
    text-align: center;
  }
</style>
