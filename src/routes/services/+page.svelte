<script lang="ts">
  import { onDestroy, onMount } from "svelte";

  import { goto } from "$app/navigation";

  import Icon from "$lib/components/ui/Icon.svelte";
  import LiveToggle from "$lib/components/ui/LiveToggle.svelte";
  import Page from "$lib/components/ui/Page.svelte";
  import PageHeader from "$lib/components/ui/PageHeader.svelte";
  import { serviceMenu } from "$lib/features/services/menu";
  import { menu } from "$lib/stores/menu.svelte";
  import ServicePanel from "$lib/features/services/ServicePanel.svelte";
  import ServiceSummary from "$lib/features/services/ServiceSummary.svelte";
  import ServiceTable from "$lib/features/services/ServiceTable.svelte";
  import { services, type Filter } from "$lib/features/services/store.svelte";

  const snapshot = $derived(services.snapshot);
  const overview = $derived(snapshot?.overview ?? null);
  const rows = $derived(services.rows);
  const selectedRow = $derived(snapshot?.services.find((s) => s.unit === services.selectedUnit));

  const filters: { key: Filter; label: string }[] = [
    { key: "all", label: "All" },
    { key: "running", label: "Running" },
    { key: "failed", label: "Failed" },
    { key: "stopped", label: "Stopped" },
    { key: "boot", label: "At boot" },
  ];

  function onkeydown(e: KeyboardEvent) {
    if (e.key !== "Escape") return;
    if (services.selectedUnit !== null) services.select(null);
    else goto("/");
  }

  onMount(services.start);
  onDestroy(services.stop);
</script>

<svelte:window {onkeydown} />

<div class="layout">
  <div class="content">
    <Page fill>
      <PageHeader back="/" backLabel="Devices" title="Services">
        {#snippet actions()}
          <label class="search">
            <Icon name="search" size={15} />
            <input
              type="search"
              aria-label="Search services"
              placeholder="Search name, description or PID"
              bind:value={services.search}
            />
          </label>
          <LiveToggle bind:live={services.live} />
        {/snippet}
      </PageHeader>

      {#if services.error && !snapshot}
        <p class="error">Couldn't read services: {services.error}</p>
      {:else if !snapshot}
        <div class="skeleton" style="height: 96px; margin-bottom: 16px"></div>
        <div class="skeleton" style="height: 420px"></div>
      {:else if !snapshot.available}
        <p class="error">This computer doesn't use systemd, so there is no service list to show.</p>
      {:else if overview}
        <ServiceSummary {overview} />

        <div class="toolbar">
          <div class="chips" role="group" aria-label="Filter services">
            {#each filters as f}
              <button
                class="chip"
                aria-pressed={services.filter === f.key}
                onclick={() => (services.filter = f.key)}
              >
                {f.label}
                <span class="tabular">{services.count(f.key)}</span>
              </button>
            {/each}
          </div>
          <p class="muted">
            {rows.length}
            {rows.length === 1 ? "service" : "services"}{services.search ? " match" : ""}
          </p>
        </div>

        <ServiceTable
          {rows}
          sortKey={services.sortKey}
          sortDesc={services.sortDesc}
          selectedUnit={services.selectedUnit}
          emptyText={services.search
            ? `No services match “${services.search}”.`
            : "No services in this group."}
          onsort={(key) => services.sortBy(key)}
          onselect={(unit) => services.select(unit)}
          oncontext={(r, e) => menu.open(e, serviceMenu(r), r.unit)}
        />
      {/if}
    </Page>
  </div>

  {#if services.selectedUnit !== null}
    <ServicePanel
      row={selectedRow}
      detail={services.detail}
      onclose={() => services.select(null)}
    />
  {/if}
</div>

<style>
  .layout {
    position: relative;
    display: flex;
    height: 100%;
  }
  .content {
    flex: 1;
    min-width: 0;
  }
  .search {
    display: flex;
    align-items: center;
    gap: var(--s-2);
    width: 300px;
    padding: 7px var(--s-3);
    border: 1px solid var(--border);
    border-radius: 0;
    background: var(--surface);
    color: var(--text-3);
  }
  .search:focus-within {
    border-color: var(--accent);
    box-shadow: var(--glow);
  }
  .search input {
    flex: 1;
    min-width: 0;
    border: 0;
    outline: 0;
    background: transparent;
    font-family: var(--font-mono);
    font-size: var(--fs-small);
  }
  .toolbar {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    justify-content: space-between;
    gap: var(--s-3);
    margin-bottom: var(--s-2);
  }
  .chips {
    display: flex;
    flex-wrap: wrap;
    gap: var(--s-2);
  }
  .chip {
    display: inline-flex;
    align-items: center;
    gap: var(--s-2);
    padding: 5px var(--s-3);
    border: 1px solid var(--border);
    background: var(--surface);
    color: var(--text-2);
    font-family: var(--font-mono);
    font-size: 11px;
    letter-spacing: 0.05em;
    text-transform: uppercase;
    cursor: pointer;
  }
  .chip span {
    color: var(--text-3);
  }
  .chip:hover {
    border-color: var(--border-strong);
    color: var(--text);
  }
  .chip[aria-pressed="true"] {
    border-color: var(--accent);
    background: var(--accent-soft);
    color: var(--accent);
    box-shadow: var(--glow);
  }
  .chip[aria-pressed="true"] span {
    color: var(--accent);
  }
  .muted {
    color: var(--text-2);
  }
  .error {
    padding: var(--s-6) 0;
    color: var(--danger);
    text-align: center;
  }
</style>
