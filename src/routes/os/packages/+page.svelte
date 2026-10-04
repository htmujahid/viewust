<script lang="ts">
  import { onMount } from "svelte";

  import { goto } from "$app/navigation";

  import Badge from "$lib/components/ui/Badge.svelte";
  import DataTable, { type Column } from "$lib/components/ui/DataTable.svelte";
  import DetailList from "$lib/components/ui/DetailList.svelte";
  import RescanButton from "$lib/components/ui/RescanButton.svelte";
  import SearchBox from "$lib/components/ui/SearchBox.svelte";
  import StatTiles from "$lib/components/ui/StatTiles.svelte";
  import { filterPackages, PACKAGE_LIMIT, packageKey } from "$lib/features/os/logic";
  import SectionShell from "$lib/components/ui/SectionShell.svelte";
  import { os } from "$lib/features/os/store.svelte";
  import { menu } from "$lib/stores/menu.svelte";
  import { groupBySection } from "$lib/utils/details";
  import { copyText } from "$lib/utils/actions";

  const data = $derived(os.packages.data);
  // The wider software picture (snaps, flatpaks, glibc, systemd); the tiles cover the rest.
  const softwareRows = $derived(
    groupBySection(os.summary.data?.details ?? [])
      .find((s) => s.title === "Software")
      ?.rows.filter((r) => r.label !== "Package manager" && r.label !== "Installed packages") ?? [],
  );
  const matches = $derived(filterPackages(data?.packages ?? [], os.packageSearch));
  const shown = $derived(matches.slice(0, PACKAGE_LIMIT));

  const columns: Column[] = [
    { key: "name", label: "Package", sortable: false },
    { key: "version", label: "Version", sortable: false },
    { key: "arch", label: "Architecture", sortable: false },
  ];

  function onkeydown(e: KeyboardEvent) {
    if (e.key === "Escape") goto("/os");
  }

  onMount(() => {
    os.packages.ensure();
    os.summary.ensure();
  });
</script>

<svelte:window {onkeydown} />

<SectionShell title="Packages">
  {#snippet actions()}
    <SearchBox
      bind:value={os.packageSearch}
      placeholder="Search name or version"
      label="Search packages"
    />
    <RescanButton loading={os.packages.loading} onclick={os.packages.load} label />
  {/snippet}

  {#if os.packages.error && !data}
    <p class="error">Couldn't read the package list: {os.packages.error}</p>
  {:else if !data}
    <div class="skeleton" style="height: 96px; margin-bottom: 16px"></div>
    <div class="skeleton" style="height: 420px"></div>
  {:else if data.manager === null}
    <p class="empty">No known package manager (apt, rpm or pacman) was found on this system.</p>
  {:else}
    <StatTiles
      items={[
        { label: "Installed", value: String(data.packages.length), sub: "packages" },
        { label: "Manager", value: data.manager },
        {
          label: "Matching",
          value: String(matches.length),
          sub: os.packageSearch ? "your search" : "all",
        },
        {
          label: "Shown",
          value: String(shown.length),
          sub: matches.length > shown.length ? `first ${PACKAGE_LIMIT}` : "all of them",
        },
      ]}
    />

    {#if softwareRows.length}
      <section class="card facts">
        <h2>Other software</h2>
        <DetailList rows={softwareRows} stackAt={64} />
      </section>
    {/if}

    {#if matches.length > shown.length}
      <p class="hint">
        Showing the first {PACKAGE_LIMIT} of {matches.length}. Search to narrow the list.
      </p>
    {/if}

    <DataTable
      rows={shown}
      {columns}
      rowKey={packageKey}
      sortKey=""
      sortDesc={false}
      emptyText="No packages match “{os.packageSearch}”."
      onsort={() => {}}
      onselect={() => {}}
      oncontext={(p, e) =>
        menu.open(
          e,
          [
            { label: "Copy name", onselect: () => copyText(p.name, "package name") },
            {
              label: "Copy name and version",
              hint: p.version,
              onselect: () => copyText(`${p.name} ${p.version}`, "package"),
            },
            { label: "Copy version", onselect: () => copyText(p.version, "version") },
          ],
          p.name,
        )}
    >
      {#snippet cell(p, c)}
        {#if c.key === "name"}
          <b>{p.name}</b>
        {:else if c.key === "version"}
          <span class="muted">{p.version}</span>
        {:else if p.arch}
          <Badge>{p.arch}</Badge>
        {:else}
          <span class="muted">—</span>
        {/if}
      {/snippet}
    </DataTable>
  {/if}
</SectionShell>

<style>
  .facts {
    margin-bottom: var(--s-4);
    padding: var(--s-4) var(--s-5);
  }
  .facts h2 {
    margin-bottom: var(--s-3);
    color: var(--accent);
    font-family: var(--font-mono);
    font-size: var(--fs-label);
    font-weight: 700;
    letter-spacing: 0.06em;
    text-transform: uppercase;
  }
  .muted {
    color: var(--text-2);
  }
  .hint {
    margin-bottom: var(--s-2);
    color: var(--text-3);
    font-size: var(--fs-small);
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
