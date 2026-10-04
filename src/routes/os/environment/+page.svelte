<script lang="ts">
  import { onMount } from "svelte";

  import { goto } from "$app/navigation";

  import Badge from "$lib/components/ui/Badge.svelte";
  import DataTable, { type Column } from "$lib/components/ui/DataTable.svelte";
  import RescanButton from "$lib/components/ui/RescanButton.svelte";
  import SearchBox from "$lib/components/ui/SearchBox.svelte";
  import StatTiles from "$lib/components/ui/StatTiles.svelte";
  import { filterEnv } from "$lib/features/os/logic";
  import SectionShell from "$lib/components/ui/SectionShell.svelte";
  import { os } from "$lib/features/os/store.svelte";
  import { menu } from "$lib/stores/menu.svelte";
  import { copyText } from "$lib/utils/actions";

  const all = $derived(os.environment.data ?? []);
  const rows = $derived(filterEnv(all, os.envSearch));

  const columns: Column[] = [
    { key: "key", label: "Name", sortable: false },
    { key: "value", label: "Value", sortable: false },
  ];

  function onkeydown(e: KeyboardEvent) {
    if (e.key === "Escape") goto("/os");
  }

  onMount(os.environment.ensure);
</script>

<svelte:window {onkeydown} />

<SectionShell title="Environment">
  {#snippet actions()}
    <SearchBox
      bind:value={os.envSearch}
      placeholder="Search name or value"
      label="Search environment"
    />
    <RescanButton loading={os.environment.loading} onclick={os.environment.load} label />
  {/snippet}

  {#if os.environment.error && !os.environment.data}
    <p class="error">Couldn't read the environment: {os.environment.error}</p>
  {:else if !os.environment.data}
    <div class="skeleton" style="height: 96px; margin-bottom: 16px"></div>
    <div class="skeleton" style="height: 320px"></div>
  {:else}
    <StatTiles
      items={[
        { label: "Variables", value: String(all.length), sub: "in this session" },
        {
          label: "Hidden",
          value: String(all.filter((e) => e.hidden).length),
          sub: "look like secrets",
        },
        { label: "Shown", value: String(rows.length), sub: os.envSearch ? "match" : "of all" },
      ]}
    />
    <p class="hint">
      These are the variables Viewust itself was started with. Values that look like passwords,
      tokens or keys are never loaded into the window.
    </p>

    <DataTable
      {rows}
      {columns}
      rowKey={(e) => e.key}
      sortKey=""
      sortDesc={false}
      emptyText="Nothing matches “{os.envSearch}”."
      onsort={() => {}}
      onselect={() => {}}
      oncontext={(e, ev) =>
        menu.open(
          ev,
          [
            { label: "Copy name", onselect: () => copyText(e.key, "variable name") },
            { label: "Copy value", disabled: e.hidden, onselect: () => copyText(e.value, "value") },
            {
              label: "Copy as NAME=value",
              disabled: e.hidden,
              onselect: () => copyText(`${e.key}=${e.value}`, "variable"),
            },
          ],
          e.key,
        )}
    >
      {#snippet cell(e, c)}
        {#if c.key === "key"}
          <b>{e.key}</b>
        {:else if e.hidden}
          <Badge tone="warn">Hidden</Badge>
        {:else}
          <span class="value muted" title={e.value}>{e.value}</span>
        {/if}
      {/snippet}
    </DataTable>
  {/if}
</SectionShell>

<style>
  .muted {
    color: var(--text-2);
  }
  .value {
    display: block;
    max-width: min(70ch, 60vw);
    overflow: hidden;
    text-overflow: ellipsis;
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
