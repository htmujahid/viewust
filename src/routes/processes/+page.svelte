<script lang="ts">
  import { onDestroy, onMount } from "svelte";

  import { goto } from "$app/navigation";

  import LiveToggle from "$lib/components/ui/LiveToggle.svelte";
  import SearchBox from "$lib/components/ui/SearchBox.svelte";
  import SectionShell from "$lib/components/ui/SectionShell.svelte";
  import ProcessPanel from "$lib/features/processes/ProcessPanel.svelte";
  import ProcessSummary from "$lib/features/processes/ProcessSummary.svelte";
  import { processMenu } from "$lib/features/processes/menu";
  import ProcessTable from "$lib/features/processes/ProcessTable.svelte";
  import { menu } from "$lib/stores/menu.svelte";
  import { processes } from "$lib/features/processes/store.svelte";

  const overview = $derived(processes.snapshot?.overview ?? null);
  const rows = $derived(processes.rows);
  const selectedRow = $derived(
    processes.snapshot?.processes.find((p) => p.pid === processes.selectedPid),
  );

  function onkeydown(e: KeyboardEvent) {
    if (e.key !== "Escape") return;
    if (processes.selectedPid !== null) processes.select(null);
    else goto("/os");
  }

  onMount(processes.start);
  onDestroy(processes.stop);
</script>

<svelte:window {onkeydown} />

<SectionShell title="Processes">
  {#snippet actions()}
    <SearchBox
      bind:value={processes.search}
      placeholder="Search name, owner or PID"
      label="Search processes"
    />
    <LiveToggle bind:live={processes.live} />
  {/snippet}

  {#if processes.error && !overview}
    <p class="error">Couldn't read processes: {processes.error}</p>
  {:else if !overview}
    <div class="skeleton" style="height: 96px; margin-bottom: 16px"></div>
    <div class="skeleton" style="height: 420px"></div>
  {:else}
    <ProcessSummary {overview} />

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

    <ProcessTable
      {rows}
      memoryTotal={overview.memory_total}
      sortKey={processes.sortKey}
      sortDesc={processes.sortDesc}
      selectedPid={processes.selectedPid}
      emptyText="No processes match “{processes.search}”."
      onsort={(key) => processes.sortBy(key)}
      onselect={(pid) => processes.select(pid)}
      oncontext={(p, e) => menu.open(e, processMenu(p), `${p.name} · ${p.pid}`)}
    />
  {/if}

  {#snippet panel()}
    {#if processes.selectedPid !== null}
      <ProcessPanel
        row={selectedRow}
        detail={processes.detail}
        onclose={() => processes.select(null)}
        onselect={(pid) => processes.select(pid)}
      />
    {/if}
  {/snippet}
</SectionShell>

<style>
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
  .error {
    padding: var(--s-6) 0;
    color: var(--danger);
    text-align: center;
  }
</style>
