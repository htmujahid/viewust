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
  const vm = osOverview.vms;
  const data = $derived(vm.data);

  const columns: Column[] = [
    { key: "name", label: "Machine", sortable: false },
    { key: "state", label: "State", sortable: false },
    { key: "manager", label: "Managed by", sortable: false },
  ];

  function onkeydown(e: KeyboardEvent) {
    if (e.key === "Escape") goto("/virtualization");
  }

  onMount(vm.ensure);
</script>

<svelte:window {onkeydown} />

<SectionShell title="Virtual machines">
  {#snippet actions()}
    <RescanButton loading={vm.loading} onclick={vm.load} label />
  {/snippet}

  {#if vm.error && !data}
    <p class="error">Couldn't read the virtual machines: {vm.error}</p>
  {:else if !data}
    <div class="skeleton" style="height: 96px; margin-bottom: 16px"></div>
    <div class="skeleton" style="height: 320px"></div>
  {:else}
    <StatTiles
      items={[
        {
          label: "Managers",
          value: data.managers.length ? data.managers.join(" · ") : "None",
          tone: data.managers.length ? undefined : "warn",
        },
        { label: "Running", value: String(data.running), tone: data.running ? "ok" : undefined },
        { label: "Defined", value: String(data.vms.length) },
      ]}
    />

    {#if data.note}
      <p class="empty">{data.note}</p>
    {:else}
      <DataTable
        rows={data.vms}
        {columns}
        rowKey={(v) => `${v.manager}:${v.name}`}
        sortKey=""
        sortDesc={false}
        emptyText="No virtual machines."
        onsort={() => {}}
        onselect={() => {}}
        oncontext={(v, e) =>
          menu.open(
            e,
            [{ label: "Copy name", onselect: () => copyText(v.name, "machine name") }],
            v.name,
          )}
      >
        {#snippet cell(v, c)}
          {#if c.key === "name"}
            <b>{v.name}</b>
          {:else if c.key === "state"}
            <Badge tone={v.state.startsWith("running") ? "ok" : "neutral"}>{v.state}</Badge>
          {:else}
            <span class="muted">{v.manager}</span>
          {/if}
        {/snippet}
      </DataTable>
    {/if}
  {/if}
</SectionShell>

<style>
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
