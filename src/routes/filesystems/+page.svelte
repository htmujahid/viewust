<script lang="ts">
  import { onDestroy, onMount } from "svelte";

  import { goto } from "$app/navigation";

  import type { FilesystemKind, FilesystemRow } from "$lib/api/types";
  import Badge from "$lib/components/ui/Badge.svelte";
  import DataTable, { type Column } from "$lib/components/ui/DataTable.svelte";
  import DetailList from "$lib/components/ui/DetailList.svelte";
  import FilterChips from "$lib/components/ui/FilterChips.svelte";
  import LiveToggle from "$lib/components/ui/LiveToggle.svelte";
  import Meter from "$lib/components/ui/Meter.svelte";
  import Page from "$lib/components/ui/Page.svelte";
  import PageHeader from "$lib/components/ui/PageHeader.svelte";
  import SearchBox from "$lib/components/ui/SearchBox.svelte";
  import SidePanel from "$lib/components/ui/SidePanel.svelte";
  import StatTiles from "$lib/components/ui/StatTiles.svelte";
  import {
    displayName,
    healthOf,
    KIND_ORDER,
    KINDS,
    usedPercent,
    type Filter,
  } from "$lib/features/filesystems/logic";
  import { filesystemMenu } from "$lib/features/filesystems/menu";
  import { menu } from "$lib/stores/menu.svelte";
  import { filesystems, type SortKey } from "$lib/features/filesystems/store.svelte";
  import { formatBytes } from "$lib/utils/format";
  import { groupBySection } from "$lib/utils/details";

  const snapshot = $derived(filesystems.snapshot);
  const overview = $derived(snapshot?.overview ?? null);
  const rows = $derived(filesystems.rows);
  const selected = $derived(filesystems.selected);
  const sections = $derived(groupBySection(selected?.details ?? []));
  const overallUsed = $derived(
    overview && overview.size > 0
      ? (overview.used / (overview.used + overview.available)) * 100
      : 0,
  );

  // One table per storage system, in the order that puts real storage first.
  const groups = $derived(
    KIND_ORDER.map((kind) => ({ kind, rows: rows.filter((r) => r.kind === kind) })).filter(
      (g) => g.rows.length > 0,
    ),
  );

  const filters = $derived<{ key: Filter; label: string; count: number }[]>([
    { key: "all", label: "All", count: filesystems.count("all") },
    { key: "local", label: "Attached", count: filesystems.count("local") },
    ...KIND_ORDER.map((kind) => ({
      key: kind as Filter,
      label: KINDS[kind].label,
      count: filesystems.count(kind),
    })),
  ]);

  const columns: Column[] = [
    { key: "mount", label: "Volume" },
    { key: "fstype", label: "Type" },
    { key: "size", label: "Size", right: true },
    { key: "used", label: "Used", hint: "Share of the usable space that is taken" },
    { key: "available", label: "Free", right: true },
  ];

  const kindBadge = (kind: FilesystemKind) =>
    kind === "removable" || kind === "network" ? ("warn" as const) : ("neutral" as const);

  function onkeydown(e: KeyboardEvent) {
    if (e.key !== "Escape") return;
    if (filesystems.selectedMount !== null) filesystems.selectedMount = null;
    else goto("/");
  }

  onMount(filesystems.start);
  onDestroy(filesystems.stop);
</script>

<svelte:window {onkeydown} />

<div class="layout">
  <div class="content">
    <Page>
      <PageHeader back="/" backLabel="Overview" title="Filesystems">
        {#snippet actions()}
          <SearchBox
            bind:value={filesystems.search}
            placeholder="Search folder, device or type"
            label="Search filesystems"
          />
          <LiveToggle bind:live={filesystems.live} />
        {/snippet}
      </PageHeader>

      {#if filesystems.error && !snapshot}
        <p class="error">Couldn't read the filesystems: {filesystems.error}</p>
      {:else if !snapshot}
        <div class="skeleton" style="height: 96px; margin-bottom: 16px"></div>
        <div class="skeleton" style="height: 420px"></div>
      {:else if overview}
        <StatTiles
          items={[
            {
              label: "Storage",
              value: formatBytes(overview.size),
              sub: `${overview.volumes} volumes`,
            },
            {
              label: "Used",
              value: formatBytes(overview.used),
              sub: `${overallUsed.toFixed(0)}% of usable`,
              tone: overallUsed >= 95 ? "danger" : overallUsed >= 85 ? "warn" : undefined,
            },
            { label: "Free", value: formatBytes(overview.available) },
            {
              label: "Mounted",
              value: String(snapshot.filesystems.length),
              sub: "filesystems in all",
            },
          ]}
        />

        <div class="toolbar">
          <FilterChips
            options={filters}
            bind:value={filesystems.filter}
            label="Filter by storage"
          />
          <p class="muted">
            {rows.length}
            {rows.length === 1 ? "filesystem" : "filesystems"}{filesystems.search ? " match" : ""}
          </p>
        </div>

        {#each groups as g (g.kind)}
          <section class="group">
            <header>
              <h2>{KINDS[g.kind].label} <span class="tabular">{g.rows.length}</span></h2>
              <p>{KINDS[g.kind].hint}</p>
            </header>
            <DataTable
              rows={g.rows}
              {columns}
              rowKey={(r: FilesystemRow) => r.mount}
              selectedKey={filesystems.selectedMount}
              sortKey={filesystems.sortKey}
              sortDesc={filesystems.sortDesc}
              emptyText=""
              onsort={(k) => filesystems.sortBy(k as SortKey)}
              onselect={(k) => (filesystems.selectedMount = k)}
              oncontext={(r, e) => menu.open(e, filesystemMenu(r), r.mount)}
            >
              {#snippet cell(r, c)}
                {@const pct = usedPercent(r)}
                {@const health = healthOf(r)}
                {#if c.key === "mount"}
                  <b class="truncate" title={r.mount}>{displayName(r)}</b>
                  <small class="truncate muted" title={r.source}>
                    {r.label ? `${r.mount} · ` : ""}{r.source}
                  </small>
                {:else if c.key === "fstype"}
                  <Badge tone={kindBadge(r.kind)}>{r.fstype}</Badge>
                  {#if r.read_only}<Badge>Read-only</Badge>{/if}
                {:else if c.key === "size"}
                  <span class="tabular">{r.size === null ? "—" : formatBytes(r.size)}</span>
                {:else if c.key === "used"}
                  {#if pct !== null}
                    <span class="tabular">{pct.toFixed(0)}%</span>
                    {#if health}<Badge tone={health.tone}>{health.label}</Badge>{/if}
                    <Meter
                      value={pct}
                      height={4}
                      tone={r.kind === "image" || r.kind === "optical" ? "accent" : undefined}
                    />
                  {:else}
                    <span class="muted">—</span>
                  {/if}
                {:else}
                  <span class="tabular"
                    >{r.available === null ? "—" : formatBytes(r.available)}</span
                  >
                {/if}
              {/snippet}
            </DataTable>
          </section>
        {:else}
          <p class="empty">
            {filesystems.search
              ? `No filesystems match “${filesystems.search}”.`
              : "Nothing is mounted in this group."}
          </p>
        {/each}
      {/if}
    </Page>
  </div>

  {#if selected}
    <SidePanel
      title={displayName(selected)}
      subtitle={selected.label ? selected.mount : selected.source}
      label="Filesystem details"
      onclose={() => (filesystems.selectedMount = null)}
    >
      {#snippet badges()}
        <Badge tone={kindBadge(selected.kind)}>{KINDS[selected.kind].label}</Badge>
        <Badge>{selected.fstype}</Badge>
        {#if selected.read_only}<Badge>Read-only</Badge>{/if}
      {/snippet}
      {#if usedPercent(selected) !== null}
        <section>
          <Meter value={usedPercent(selected) ?? 0} />
        </section>
      {/if}
      {#each sections as s}
        <section>
          <h3>{s.title}</h3>
          <DetailList rows={s.rows} stackAt={40} />
        </section>
      {/each}
    </SidePanel>
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
  .toolbar {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    justify-content: space-between;
    gap: var(--s-3);
    margin-bottom: var(--s-4);
  }
  .group {
    margin-bottom: var(--s-5);
  }
  .group :global(.wrap) {
    overflow: visible;
    min-height: 0;
  }
  .group header {
    display: flex;
    flex-wrap: wrap;
    align-items: baseline;
    gap: var(--s-2) var(--s-3);
    margin-bottom: var(--s-2);
  }
  h2 {
    font-size: var(--fs-label);
    font-weight: 600;
    letter-spacing: 0.06em;
    text-transform: uppercase;
  }
  h2 span {
    color: var(--text-3);
  }
  .group header p,
  .muted {
    color: var(--text-2);
  }
  :global(td) b,
  :global(td) small {
    display: block;
    max-width: 380px;
  }
  :global(td) small {
    font-family: var(--font-ui);
    font-size: var(--fs-label);
  }
  section :global(.meter) {
    margin-top: var(--s-3);
  }
  section :global(td .meter) {
    margin-top: 3px;
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
