<script lang="ts">
  import { onMount } from "svelte";

  import { goto } from "$app/navigation";

  import Badge from "$lib/components/ui/Badge.svelte";
  import DataTable, { type Column } from "$lib/components/ui/DataTable.svelte";
  import RescanButton from "$lib/components/ui/RescanButton.svelte";
  import SearchBox from "$lib/components/ui/SearchBox.svelte";
  import SectionShell from "$lib/components/ui/SectionShell.svelte";
  import StatTiles from "$lib/components/ui/StatTiles.svelte";
  import { osOverview } from "$lib/features/os/overview.svelte";
  import { menu } from "$lib/stores/menu.svelte";
  import { copyText } from "$lib/utils/actions";

  // A stable handle: the Loaded instance itself never changes, only its fields do.
  const conn = osOverview.connections;
  const data = $derived(conn.data);

  let search = $state("");
  const rows = $derived(
    (data?.rows ?? []).filter((r) => {
      const q = search.trim().toLowerCase();
      return !q || [r.local, r.remote, r.state, r.proto].some((v) => v.toLowerCase().includes(q));
    }),
  );

  const columns: Column[] = [
    { key: "proto", label: "Protocol", sortable: false },
    { key: "local", label: "This computer", sortable: false },
    { key: "remote", label: "The other end", sortable: false },
    { key: "state", label: "State", sortable: false },
  ];

  const tone = (state: string) =>
    state === "established"
      ? ("ok" as const)
      : state === "listening"
        ? ("neutral" as const)
        : ("warn" as const);

  function onkeydown(e: KeyboardEvent) {
    if (e.key === "Escape") goto("/os");
  }

  onMount(conn.ensure);
</script>

<svelte:window {onkeydown} />

<SectionShell title="Connections">
  {#snippet actions()}
    <SearchBox
      bind:value={search}
      placeholder="Search address, port or state"
      label="Search connections"
    />
    <RescanButton loading={conn.loading} onclick={conn.load} label />
  {/snippet}

  {#if conn.error && !data}
    <p class="error">Couldn't read the connections: {conn.error}</p>
  {:else if !data}
    <div class="skeleton" style="height: 96px; margin-bottom: 16px"></div>
    <div class="skeleton" style="height: 420px"></div>
  {:else}
    <StatTiles
      items={[
        { label: "Established", value: String(data.established), sub: "talking right now" },
        { label: "Listening", value: String(data.listening), sub: "waiting for callers" },
        { label: "Closing down", value: String(data.time_wait), sub: "recently finished" },
        { label: "Shown", value: String(rows.length), sub: search ? "match" : "of all" },
      ]}
    />

    <DataTable
      {rows}
      {columns}
      rowKey={(r) => `${r.proto} ${r.local} ${r.remote}`}
      sortKey=""
      sortDesc={false}
      emptyText="No connections match “{search}”."
      onsort={() => {}}
      onselect={() => {}}
      oncontext={(r, e) =>
        menu.open(
          e,
          [
            {
              label: "Copy remote address",
              hint: r.remote,
              onselect: () => copyText(r.remote, "address"),
            },
            { label: "Copy local address", onselect: () => copyText(r.local, "address") },
          ],
          `${r.local} → ${r.remote}`,
        )}
    >
      {#snippet cell(r, c)}
        {#if c.key === "proto"}
          <span class="muted">{r.proto}</span>
        {:else if c.key === "local"}
          <span class="tabular">{r.local}</span>
        {:else if c.key === "remote"}
          <span class="tabular"
            >{r.remote === "0.0.0.0:0" || r.remote === "[::]:0" ? "—" : r.remote}</span
          >
        {:else}
          <Badge tone={tone(r.state)}>{r.state}</Badge>
        {/if}
      {/snippet}
    </DataTable>
  {/if}
</SectionShell>

<style>
  .muted {
    color: var(--text-2);
  }
  .error {
    padding: var(--s-6) 0;
    color: var(--danger);
    text-align: center;
  }
</style>
