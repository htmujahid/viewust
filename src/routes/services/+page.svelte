<script lang="ts">
  import { onDestroy, onMount } from "svelte";

  import { goto } from "$app/navigation";

  import LiveToggle from "$lib/components/ui/LiveToggle.svelte";
  import SearchBox from "$lib/components/ui/SearchBox.svelte";
  import SectionShell from "$lib/components/ui/SectionShell.svelte";
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
    else goto("/os");
  }

  onMount(services.start);
  onDestroy(services.stop);
</script>

<svelte:window {onkeydown} />

<SectionShell title="Services">
  {#snippet actions()}
    <SearchBox
      bind:value={services.search}
      placeholder="Search name, description or PID"
      label="Search services"
    />
    <LiveToggle bind:live={services.live} />
  {/snippet}

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

  {#snippet panel()}
    {#if services.selectedUnit !== null}
      <ServicePanel
        row={selectedRow}
        detail={services.detail}
        onclose={() => services.select(null)}
      />
    {/if}
  {/snippet}
</SectionShell>

<style>
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
