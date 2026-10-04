<script lang="ts">
  import { onMount } from "svelte";

  import { goto } from "$app/navigation";

  import type { KernelModule } from "$lib/api/types";
  import Badge from "$lib/components/ui/Badge.svelte";
  import DataTable, { type Column } from "$lib/components/ui/DataTable.svelte";
  import DetailList from "$lib/components/ui/DetailList.svelte";
  import RescanButton from "$lib/components/ui/RescanButton.svelte";
  import SearchBox from "$lib/components/ui/SearchBox.svelte";
  import SidePanel from "$lib/components/ui/SidePanel.svelte";
  import StatTiles from "$lib/components/ui/StatTiles.svelte";
  import { filterModules } from "$lib/features/os/logic";
  import OsShell from "$lib/features/os/OsShell.svelte";
  import { os } from "$lib/features/os/store.svelte";
  import { menu } from "$lib/stores/menu.svelte";
  import { copyText } from "$lib/utils/actions";
  import { groupBySection } from "$lib/utils/details";
  import { formatBytes } from "$lib/utils/format";
  import { sortRows } from "$lib/utils/sort";

  const all = $derived(os.modules.data ?? []);
  const rows = $derived(
    sortRows(
      filterModules(all, os.moduleSearch),
      (m: KernelModule) =>
        os.moduleSort === "name" ? m.name : os.moduleSort === "used" ? m.used_by.length : m.size,
      os.moduleDesc,
      (m) => m.name,
    ),
  );
  const sections = $derived(groupBySection(os.moduleInfo?.details ?? []));
  const total = $derived(all.reduce((n, m) => n + m.size, 0));

  const columns: Column[] = [
    { key: "name", label: "Module" },
    { key: "size", label: "Memory", right: true },
    { key: "used", label: "Used by", hint: "Other modules that rely on this one" },
  ];

  function onkeydown(e: KeyboardEvent) {
    if (e.key !== "Escape") return;
    if (os.selectedModule !== null) os.selectModule(null);
    else goto("/");
  }

  onMount(os.modules.ensure);
</script>

<svelte:window {onkeydown} />

<OsShell>
  {#snippet actions()}
    <SearchBox
      bind:value={os.moduleSearch}
      placeholder="Search module or user"
      label="Search kernel modules"
    />
    <RescanButton loading={os.modules.loading} onclick={os.modules.load} label />
  {/snippet}

  {#if os.modules.error && !os.modules.data}
    <p class="error">Couldn't read the kernel modules: {os.modules.error}</p>
  {:else if !os.modules.data}
    <div class="skeleton" style="height: 96px; margin-bottom: 16px"></div>
    <div class="skeleton" style="height: 420px"></div>
  {:else}
    <StatTiles
      items={[
        { label: "Loaded", value: String(all.length), sub: "kernel modules" },
        { label: "Memory", value: formatBytes(total) },
        {
          label: "Unused",
          value: String(all.filter((m) => m.used_by.length === 0).length),
          sub: "no other module needs them",
        },
        { label: "Shown", value: String(rows.length), sub: os.moduleSearch ? "match" : "of all" },
      ]}
    />

    <DataTable
      {rows}
      {columns}
      rowKey={(m) => m.name}
      selectedKey={os.selectedModule}
      sortKey={os.moduleSort}
      sortDesc={os.moduleDesc}
      emptyText="No modules match “{os.moduleSearch}”."
      onsort={(k) => os.sortModules(k)}
      onselect={(k) => os.selectModule(k)}
      oncontext={(m, e) =>
        menu.open(
          e,
          [
            { label: "View details", onselect: () => os.selectModule(m.name) },
            { label: "Copy name", onselect: () => copyText(m.name, "module name") },
          ],
          m.name,
        )}
    >
      {#snippet cell(m, c)}
        {#if c.key === "name"}
          <b>{m.name}</b>
        {:else if c.key === "size"}
          <span class="tabular">{formatBytes(m.size)}</span>
        {:else if m.used_by.length === 0}
          <span class="muted">—</span>
        {:else}
          <span class="muted">{m.used_by.slice(0, 3).join(", ")}</span>
          {#if m.used_by.length > 3}<Badge>+{m.used_by.length - 3}</Badge>{/if}
        {/if}
      {/snippet}
    </DataTable>
  {/if}

  {#snippet panel()}
    {#if os.selectedModule !== null}
      <SidePanel
        title={os.selectedModule}
        subtitle="Kernel module"
        label="Module details"
        onclose={() => os.selectModule(null)}
      >
        {#if !os.moduleInfo}
          <p class="muted pad">Reading…</p>
        {:else if !os.moduleInfo.found}
          <p class="muted pad">No information about this module.</p>
        {:else}
          {#each sections as s}
            <section>
              <h3>{s.title}</h3>
              <DetailList rows={s.rows} stackAt={40} />
            </section>
          {/each}
        {/if}
      </SidePanel>
    {/if}
  {/snippet}
</OsShell>

<style>
  .muted {
    color: var(--text-2);
  }
  .pad {
    padding: var(--s-5) 0;
    text-align: center;
  }
  .error {
    padding: var(--s-6) 0;
    color: var(--danger);
    text-align: center;
  }
</style>
