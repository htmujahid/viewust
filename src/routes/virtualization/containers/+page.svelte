<script lang="ts">
  import { onMount } from "svelte";

  import { goto } from "$app/navigation";

  import Badge from "$lib/components/ui/Badge.svelte";
  import DataTable, { type Column } from "$lib/components/ui/DataTable.svelte";
  import RescanButton from "$lib/components/ui/RescanButton.svelte";
  import SectionShell from "$lib/components/ui/SectionShell.svelte";
  import StatTiles from "$lib/components/ui/StatTiles.svelte";
  import { osOverview } from "$lib/features/os/overview.svelte";
  import { menu } from "$lib/stores/menu.svelte";
  import { copyText } from "$lib/utils/actions";

  // A stable handle: the Loaded instance itself never changes, only its fields do.
  const ct = osOverview.containers;
  const data = $derived(ct.data);

  const columns: Column[] = [
    { key: "name", label: "Container", sortable: false },
    { key: "image", label: "Image", sortable: false },
    { key: "state", label: "State", sortable: false },
    { key: "status", label: "Status", sortable: false },
  ];

  function onkeydown(e: KeyboardEvent) {
    if (e.key === "Escape") goto("/virtualization");
  }

  onMount(ct.ensure);
</script>

<svelte:window {onkeydown} />

<SectionShell scroll title="Containers">
  {#snippet actions()}
    <RescanButton loading={ct.loading} onclick={ct.load} label />
  {/snippet}

  {#if ct.error && !data}
    <p class="error">Couldn't read the containers: {ct.error}</p>
  {:else if !data}
    <div class="skeleton" style="height: 96px; margin-bottom: 16px"></div>
    <div class="skeleton" style="height: 320px"></div>
  {:else if data.runtime === null || data.note}
    <p class="empty">{data.note}</p>
  {:else}
    <StatTiles
      items={[
        { label: "Runtime", value: data.runtime },
        { label: "Running", value: String(data.running), tone: data.running ? "ok" : undefined },
        { label: "Stopped", value: String(data.containers.length - data.running) },
        { label: "Images", value: String(data.images.length) },
      ]}
    />

    <div class="card table">
      <DataTable
        rows={data.containers}
        {columns}
        rowKey={(c) => c.name}
        sortKey=""
        sortDesc={false}
        emptyText="No containers exist."
        onsort={() => {}}
        onselect={() => {}}
        oncontext={(c, e) =>
          menu.open(
            e,
            [
              { label: "Copy name", onselect: () => copyText(c.name, "container name") },
              { label: "Copy image", hint: c.image, onselect: () => copyText(c.image, "image") },
            ],
            c.name,
          )}
      >
        {#snippet cell(c, col)}
          {#if col.key === "name"}
            <b>{c.name}</b>
          {:else if col.key === "image"}
            <span class="muted">{c.image}</span>
          {:else if col.key === "state"}
            <Badge tone={c.state === "running" ? "ok" : "neutral"}>{c.state}</Badge>
          {:else}
            <span class="muted">{c.status}</span>
          {/if}
        {/snippet}
      </DataTable>
    </div>

    {#if data.images.length}
      <section class="card block">
        <h2>Images on disk</h2>
        <ul>
          {#each data.images as img (img.name)}
            <li><span class="truncate">{img.name}</span><span class="tabular">{img.size}</span></li>
          {/each}
        </ul>
      </section>
    {/if}
  {/if}
</SectionShell>

<style>
  .table {
    margin-bottom: var(--s-5);
  }
  .table :global(.wrap) {
    overflow: visible;
    min-height: 0;
    border: 0;
  }
  .block {
    padding: var(--s-4) var(--s-5);
  }
  .block h2 {
    margin-bottom: var(--s-3);
    color: var(--accent);
    font-family: var(--font-mono);
    font-size: var(--fs-label);
    font-weight: 700;
    letter-spacing: 0.06em;
    text-transform: uppercase;
  }
  ul {
    margin: 0;
    padding: 0;
    list-style: none;
  }
  li {
    display: flex;
    justify-content: space-between;
    gap: var(--s-4);
    padding: 6px 0;
    border-bottom: 1px solid var(--border);
    font-family: var(--font-mono);
    font-size: var(--fs-small);
  }
  li:last-child {
    border-bottom: 0;
  }
  .muted {
    color: var(--text-2);
  }
  .empty,
  .error {
    padding: var(--s-6) 0;
    text-align: center;
  }
  .empty {
    color: var(--text-3);
  }
  .error {
    color: var(--danger);
  }
</style>
