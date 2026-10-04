<script lang="ts">
  import { onMount } from "svelte";

  import { goto } from "$app/navigation";

  import DataTable, { type Column } from "$lib/components/ui/DataTable.svelte";
  import RescanButton from "$lib/components/ui/RescanButton.svelte";
  import SectionShell from "$lib/components/ui/SectionShell.svelte";
  import StatTiles from "$lib/components/ui/StatTiles.svelte";
  import { osOverview } from "$lib/features/os/overview.svelte";
  import { menu } from "$lib/stores/menu.svelte";
  import { copyText } from "$lib/utils/actions";
  import { formatBytes } from "$lib/utils/format";

  // A stable handle: the Loaded instance itself never changes, only its fields do.
  const cg = osOverview.cgroups;
  const data = $derived(cg.data);

  const columns: Column[] = [
    { key: "name", label: "Group", sortable: false },
    {
      key: "pids",
      label: "Processes",
      right: true,
      hint: "Processes in it and everything beneath it",
      sortable: false,
    },
    {
      key: "memory",
      label: "Memory",
      right: true,
      hint: "Memory charged to this group",
      sortable: false,
    },
    { key: "groups", label: "Subgroups", right: true, sortable: false },
  ];

  function onkeydown(e: KeyboardEvent) {
    if (e.key === "Escape") goto("/os");
  }

  onMount(cg.ensure);
</script>

<svelte:window {onkeydown} />

<SectionShell title="Control groups">
  {#snippet actions()}
    <RescanButton loading={cg.loading} onclick={cg.load} label />
  {/snippet}

  {#if cg.error && !data}
    <p class="error">Couldn't read the control groups: {cg.error}</p>
  {:else if !data}
    <div class="skeleton" style="height: 96px; margin-bottom: 16px"></div>
    <div class="skeleton" style="height: 320px"></div>
  {:else}
    <StatTiles
      items={[
        { label: "Version", value: data.version },
        { label: "Groups", value: data.capped ? `over ${data.groups}` : String(data.groups) },
        {
          label: "Controllers",
          value: String(data.controllers.length),
          sub: "resources they can limit",
        },
        { label: "Top-level", value: String(data.top.length), sub: "slices and scopes" },
      ]}
    />
    {#if data.controllers.length}
      <p class="hint">Controllers: {data.controllers.join(", ")}</p>
    {/if}

    <DataTable
      rows={data.top}
      {columns}
      rowKey={(r) => r.name}
      sortKey=""
      sortDesc={false}
      emptyText="No groups under the root."
      onsort={() => {}}
      onselect={() => {}}
      oncontext={(r, e) =>
        menu.open(
          e,
          [{ label: "Copy name", onselect: () => copyText(r.name, "group name") }],
          r.name,
        )}
    >
      {#snippet cell(r, c)}
        {#if c.key === "name"}
          <b>{r.name}</b>
        {:else if c.key === "pids"}
          <span class="tabular muted">{r.pids ?? "—"}</span>
        {:else if c.key === "memory"}
          <span class="tabular">{r.memory === null ? "—" : formatBytes(r.memory)}</span>
        {:else}
          <span class="tabular muted">{r.groups ?? "—"}</span>
        {/if}
      {/snippet}
    </DataTable>
  {/if}
</SectionShell>

<style>
  .muted {
    color: var(--text-2);
  }
  .hint {
    margin-bottom: var(--s-2);
    color: var(--text-3);
    font-size: var(--fs-small);
  }
  .error {
    padding: var(--s-6) 0;
    color: var(--danger);
    text-align: center;
  }
</style>
