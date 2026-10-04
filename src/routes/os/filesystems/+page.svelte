<script lang="ts">
  import { onMount } from "svelte";

  import { goto } from "$app/navigation";

  import Badge from "$lib/components/ui/Badge.svelte";
  import DataTable, { type Column } from "$lib/components/ui/DataTable.svelte";
  import Meter from "$lib/components/ui/Meter.svelte";
  import RescanButton from "$lib/components/ui/RescanButton.svelte";
  import SearchBox from "$lib/components/ui/SearchBox.svelte";
  import SectionShell from "$lib/components/ui/SectionShell.svelte";
  import StatTiles from "$lib/components/ui/StatTiles.svelte";
  import { osOverview } from "$lib/features/os/overview.svelte";
  import { menu } from "$lib/stores/menu.svelte";
  import { copyText } from "$lib/utils/actions";
  import { formatBytes } from "$lib/utils/format";

  // A stable handle: the Loaded instance itself never changes, only its fields do.
  const fs = osOverview.filesystems;
  const data = $derived(fs.data);

  let search = $state("");
  const rows = $derived(
    (data?.rows ?? []).filter((r) => {
      const q = search.trim().toLowerCase();
      return !q || [r.mount, r.source, r.fstype].some((v) => v.toLowerCase().includes(q));
    }),
  );

  const percent = (used: number | null, available: number | null) => {
    if (used === null || available === null || used + available === 0) return null;
    return (used / (used + available)) * 100;
  };

  const columns: Column[] = [
    { key: "mount", label: "Mounted at", sortable: false },
    { key: "fstype", label: "Type", sortable: false },
    { key: "size", label: "Size", right: true, sortable: false },
    {
      key: "used",
      label: "Used",
      sortable: false,
      hint: "Share of the usable space that is taken",
    },
    { key: "available", label: "Free", right: true, sortable: false },
  ];

  function onkeydown(e: KeyboardEvent) {
    if (e.key === "Escape") goto("/os");
  }

  onMount(fs.ensure);
</script>

<svelte:window {onkeydown} />

<SectionShell title="File systems">
  {#snippet actions()}
    <SearchBox
      bind:value={search}
      placeholder="Search mount, device or type"
      label="Search filesystems"
    />
    <RescanButton loading={fs.loading} onclick={fs.load} label />
  {/snippet}

  {#if fs.error && !data}
    <p class="error">Couldn't read the filesystems: {fs.error}</p>
  {:else if !data}
    <div class="skeleton" style="height: 96px; margin-bottom: 16px"></div>
    <div class="skeleton" style="height: 420px"></div>
  {:else}
    <StatTiles
      items={[
        { label: "Mounted", value: String(data.mounted), sub: "filesystems" },
        { label: "With real space", value: String(data.real), sub: "on drives or in RAM" },
        {
          label: "Kernel views",
          value: String(data.mounted - data.real),
          sub: "no space of their own",
        },
        { label: "Shown", value: String(rows.length), sub: search ? "match" : "of all" },
      ]}
    />

    <DataTable
      {rows}
      {columns}
      rowKey={(r) => r.mount}
      sortKey=""
      sortDesc={false}
      emptyText="No filesystems match “{search}”."
      onsort={() => {}}
      onselect={() => {}}
      oncontext={(r, e) =>
        menu.open(
          e,
          [
            { label: "Copy mount point", onselect: () => copyText(r.mount, "mount point") },
            { label: "Copy source", hint: r.source, onselect: () => copyText(r.source, "source") },
          ],
          r.mount,
        )}
    >
      {#snippet cell(r, c)}
        {@const pct = percent(r.used, r.available)}
        {#if c.key === "mount"}
          <b class="truncate" title={r.mount}>{r.mount}</b>
          <small class="muted truncate" title={r.source}>{r.source}</small>
        {:else if c.key === "fstype"}
          <Badge tone={r.pseudo ? "neutral" : "ok"}>{r.fstype}</Badge>
        {:else if c.key === "size"}
          <span class="tabular">{r.size === null ? "—" : formatBytes(r.size)}</span>
        {:else if c.key === "used"}
          {#if pct !== null}
            <span class="tabular">{pct.toFixed(0)}%</span>
            <Meter value={pct} height={4} />
          {:else}
            <span class="muted">—</span>
          {/if}
        {:else}
          <span class="tabular">{r.available === null ? "—" : formatBytes(r.available)}</span>
        {/if}
      {/snippet}
    </DataTable>
  {/if}
</SectionShell>

<style>
  .muted {
    color: var(--text-2);
  }
  small {
    display: block;
    font-family: var(--font-ui);
    font-size: var(--fs-label);
  }
  b {
    display: block;
    max-width: 380px;
  }
  .error {
    padding: var(--s-6) 0;
    color: var(--danger);
    text-align: center;
  }
</style>
