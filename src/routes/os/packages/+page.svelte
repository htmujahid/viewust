<script lang="ts">
  import { onMount } from "svelte";

  import { goto } from "$app/navigation";

  import Badge from "$lib/components/ui/Badge.svelte";
  import DataTable, { type Column } from "$lib/components/ui/DataTable.svelte";
  import RescanButton from "$lib/components/ui/RescanButton.svelte";
  import SearchBox from "$lib/components/ui/SearchBox.svelte";
  import StatTiles from "$lib/components/ui/StatTiles.svelte";
  import { filterPackages, PACKAGE_LIMIT, packageKey } from "$lib/features/os/logic";
  import OsShell from "$lib/features/os/OsShell.svelte";
  import { os } from "$lib/features/os/store.svelte";
  import { menu } from "$lib/stores/menu.svelte";
  import { copyText } from "$lib/utils/actions";

  const data = $derived(os.packages.data);
  const matches = $derived(filterPackages(data?.packages ?? [], os.packageSearch));
  const shown = $derived(matches.slice(0, PACKAGE_LIMIT));

  const columns: Column[] = [
    { key: "name", label: "Package", sortable: false },
    { key: "version", label: "Version", sortable: false },
    { key: "arch", label: "Architecture", sortable: false },
  ];

  function onkeydown(e: KeyboardEvent) {
    if (e.key === "Escape") goto("/");
  }

  onMount(os.packages.ensure);
</script>

<svelte:window {onkeydown} />

<OsShell>
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
</OsShell>

<style>
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
