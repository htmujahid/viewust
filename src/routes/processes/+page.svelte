<script lang="ts">
  import { onDestroy, onMount } from "svelte";

  import { goto } from "$app/navigation";

  import Icon from "$lib/components/ui/Icon.svelte";
  import LiveToggle from "$lib/components/ui/LiveToggle.svelte";
  import NavLinks from "$lib/components/ui/NavLinks.svelte";
  import Page from "$lib/components/ui/Page.svelte";
  import PageHeader from "$lib/components/ui/PageHeader.svelte";
  import ProcessPanel from "$lib/features/processes/ProcessPanel.svelte";
  import ProcessSummary from "$lib/features/processes/ProcessSummary.svelte";
  import ProcessTable from "$lib/features/processes/ProcessTable.svelte";
  import { processes } from "$lib/features/processes/store.svelte";

  const overview = $derived(processes.snapshot?.overview ?? null);
  const rows = $derived(processes.rows);
  const selectedRow = $derived(
    processes.snapshot?.processes.find((p) => p.pid === processes.selectedPid),
  );

  function onkeydown(e: KeyboardEvent) {
    if (e.key !== "Escape") return;
    if (processes.selectedPid !== null) processes.select(null);
    else goto("/");
  }

  onMount(processes.start);
  onDestroy(processes.stop);
</script>

<svelte:window {onkeydown} />

<div class="layout">
  <div class="content">
    <Page maxWidth={1600}>
      <PageHeader
        back="/"
        backLabel="Devices"
        title="Processes"
        subtitle={overview
          ? `${overview.processes} running · ${overview.threads} threads`
          : "Reading…"}
      >
        {#snippet actions()}
          <label class="search">
            <Icon name="search" size={15} />
            <input
              type="search"
              placeholder="Search name, owner or PID"
              bind:value={processes.search}
            />
          </label>
          <NavLinks hide={["processes"]} />
          <LiveToggle bind:live={processes.live} />
        {/snippet}
      </PageHeader>

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
        />
      {/if}
    </Page>
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
  .content {
    flex: 1;
    min-width: 0;
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
