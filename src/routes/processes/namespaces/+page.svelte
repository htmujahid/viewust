<script lang="ts">
  import { onDestroy, onMount } from "svelte";

  import { goto } from "$app/navigation";

  import Badge from "$lib/components/ui/Badge.svelte";
  import DataTable, { type Column } from "$lib/components/ui/DataTable.svelte";
  import DetailList from "$lib/components/ui/DetailList.svelte";
  import FilterChips from "$lib/components/ui/FilterChips.svelte";
  import LiveToggle from "$lib/components/ui/LiveToggle.svelte";
  import SearchBox from "$lib/components/ui/SearchBox.svelte";
  import SidePanel from "$lib/components/ui/SidePanel.svelte";
  import StatTiles from "$lib/components/ui/StatTiles.svelte";
  import ProcessesShell from "$lib/features/processes/ProcessesShell.svelte";
  import { menu } from "$lib/stores/menu.svelte";
  import { copyText } from "$lib/utils/actions";
  import { NS_KINDS, nsLabel } from "$lib/features/namespaces/logic";
  import { keyOf, namespaces } from "$lib/features/namespaces/store.svelte";
  import type { NamespaceRow } from "$lib/api/types";

  const snapshot = $derived(namespaces.snapshot);
  const rows = $derived(namespaces.rows);
  const selected = $derived(namespaces.find(namespaces.selected));

  const filters = $derived([
    { key: "all", label: "All", count: namespaces.count("all") },
    ...Object.keys(NS_KINDS).map((k) => ({ key: k, label: k, count: namespaces.count(k) })),
  ]);

  const columns: Column[] = [
    { key: "kind", label: "Type" },
    { key: "id", label: "ID", right: true },
    { key: "processes", label: "Programs", right: true, hint: "Programs read inside it" },
    { key: "scope", label: "Scope", hint: "Whether this app lives in it too" },
    { key: "example", label: "Example", sortable: false },
  ];

  const example = (n: NamespaceRow) =>
    n.sample[0] ? `${n.sample[0].name}${n.processes > 1 ? ` +${n.processes - 1}` : ""}` : "—";

  const distinctKinds = $derived(new Set((snapshot?.namespaces ?? []).map((n) => n.kind)).size);
  const isolated = $derived((snapshot?.namespaces ?? []).filter((n) => !n.current).length);

  function onkeydown(e: KeyboardEvent) {
    if (e.key !== "Escape") return;
    if (namespaces.selected !== null) namespaces.selected = null;
    else goto("/");
  }

  onMount(namespaces.start);
  onDestroy(namespaces.stop);
</script>

<svelte:window {onkeydown} />

<ProcessesShell>
  {#snippet actions()}
    <SearchBox
      bind:value={namespaces.search}
      placeholder="Search type, ID or program"
      label="Search namespaces"
    />
    <LiveToggle bind:live={namespaces.live} />
  {/snippet}

  {#if namespaces.error && !snapshot}
    <p class="error">Couldn't read the namespaces: {namespaces.error}</p>
  {:else if !snapshot}
    <div class="skeleton" style="height: 96px; margin-bottom: 16px"></div>
    <div class="skeleton" style="height: 420px"></div>
  {:else}
    <StatTiles
      items={[
        { label: "Namespaces", value: String(snapshot.namespaces.length) },
        { label: "Isolated", value: String(isolated), sub: "apart from this app" },
        { label: "Types in use", value: String(distinctKinds), sub: "of 8" },
        {
          label: "Programs read",
          value: String(snapshot.inspected),
          sub: `of ${snapshot.total}`,
        },
      ]}
    />

    {#if snapshot.note}<p class="note">{snapshot.note}</p>{/if}

    <div class="toolbar">
      <FilterChips options={filters} bind:value={namespaces.filter} label="Filter by type" />
      <p class="muted">{rows.length} {rows.length === 1 ? "namespace" : "namespaces"}</p>
    </div>

    <DataTable
      {rows}
      {columns}
      rowKey={keyOf}
      selectedKey={namespaces.selected}
      sortKey={namespaces.sortKey}
      sortDesc={namespaces.sortDesc}
      emptyText="No namespaces match."
      onsort={(k) => namespaces.sortBy(k)}
      onselect={(k) => (namespaces.selected = k)}
      oncontext={(n, e) =>
        menu.open(
          e,
          [
            { label: "View details", onselect: () => (namespaces.selected = keyOf(n)) },
            {
              label: "Copy ID",
              hint: String(n.id),
              onselect: () => copyText(String(n.id), "namespace ID"),
            },
          ],
          `${nsLabel(n.kind)} namespace`,
        )}
    >
      {#snippet cell(n, c)}
        {#if c.key === "kind"}
          <b>{nsLabel(n.kind)}</b> <span class="muted">{n.kind}</span>
        {:else if c.key === "id"}
          <span class="muted">{n.id}</span>
        {:else if c.key === "processes"}
          {n.processes}
        {:else if c.key === "scope"}
          <Badge tone={n.current ? "ok" : "neutral"}>{n.current ? "This app" : "Isolated"}</Badge>
        {:else}
          <span class="muted">{example(n)}</span>
        {/if}
      {/snippet}
    </DataTable>
  {/if}

  {#snippet panel()}
    {#if selected}
      <SidePanel
        title={nsLabel(selected.kind)}
        subtitle={NS_KINDS[selected.kind]?.what ?? null}
        label="Namespace details"
        onclose={() => (namespaces.selected = null)}
      >
        {#snippet badges()}
          <Badge tone={selected.current ? "ok" : "neutral"}>
            {selected.current ? "This app" : "Isolated"}
          </Badge>
          <Badge>{selected.kind}</Badge>
        {/snippet}
        <section>
          <h3>Namespace</h3>
          <DetailList
            rows={[
              { label: "Type", value: selected.kind },
              { label: "ID", value: String(selected.id) },
              { label: "Programs inside", value: String(selected.processes) },
            ]}
          />
        </section>
        <section>
          <h3>Programs inside</h3>
          <ul class="procs">
            {#each selected.sample as p (p.pid)}
              <li>
                <span class="truncate">{p.name || "(unnamed)"}</span>
                <small class="tabular">{p.pid}{p.user ? ` · ${p.user}` : ""}</small>
              </li>
            {/each}
          </ul>
          {#if selected.processes > selected.sample.length}
            <p class="muted more">
              and {selected.processes - selected.sample.length} more
            </p>
          {/if}
        </section>
      </SidePanel>
    {/if}
  {/snippet}
</ProcessesShell>

<style>
  .toolbar {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    justify-content: space-between;
    gap: var(--s-3);
    margin-bottom: var(--s-2);
  }
  .muted {
    color: var(--text-2);
  }
  .note {
    margin-bottom: var(--s-3);
    padding: var(--s-2) var(--s-3);
    border: 1px solid var(--border);
    border-left: 2px solid var(--warn);
    background: var(--warn-soft);
    color: var(--warn);
    font-size: var(--fs-small);
  }
  .procs {
    margin: 0;
    padding: 0;
    list-style: none;
  }
  .procs li {
    display: flex;
    justify-content: space-between;
    gap: var(--s-3);
    padding: 6px 0;
    border-bottom: 1px solid var(--border);
    font-family: var(--font-mono);
    font-size: var(--fs-small);
  }
  .procs small {
    flex: none;
    color: var(--text-3);
  }
  .more {
    margin-top: var(--s-2);
    font-size: var(--fs-small);
  }
  .error {
    padding: var(--s-6) 0;
    color: var(--danger);
    text-align: center;
  }
</style>
