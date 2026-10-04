<script lang="ts">
  import { onMount } from "svelte";

  import { goto } from "$app/navigation";

  import RescanButton from "$lib/components/ui/RescanButton.svelte";
  import Page from "$lib/components/ui/Page.svelte";
  import PageHeader from "$lib/components/ui/PageHeader.svelte";
  import StatTiles from "$lib/components/ui/StatTiles.svelte";
  import DiskTree from "$lib/features/disk-usage/DiskTree.svelte";
  import { menuTitle, treeMenu } from "$lib/features/disk-usage/menu";
  import { diskUsage } from "$lib/features/disk-usage/store.svelte";
  import { menu } from "$lib/stores/menu.svelte";
  import { formatBytes } from "$lib/utils/format";

  const snapshot = $derived(diskUsage.snapshot);
  const overview = $derived(snapshot?.overview ?? null);
  const rows = $derived(diskUsage.rows);
  const busy = $derived(
    diskUsage.loading || [...diskUsage.nodes.values()].some((n) => n.status === "loading"),
  );
  const usedShare = $derived(
    overview && overview.used + overview.available > 0
      ? (overview.used / (overview.used + overview.available)) * 100
      : 0,
  );

  function onkeydown(e: KeyboardEvent) {
    if (e.key === "Escape") goto("/");
  }

  // The device list is read once; opening a folder is what does the measuring.
  onMount(() => {
    if (!diskUsage.snapshot) void diskUsage.load();
  });
</script>

<svelte:window {onkeydown} />

<Page>
  <PageHeader back="/" backLabel="Overview" title="Disk usage">
    {#snippet actions()}
      <label class="check">
        <input type="checkbox" bind:checked={diskUsage.showFiles} />
        Show files
      </label>
      <button
        class="btn"
        onclick={() => diskUsage.collapseAll()}
        disabled={diskUsage.expanded.size === 0}
      >
        Collapse all
      </button>
      <RescanButton loading={busy} onclick={() => diskUsage.rescanAll()} label />
    {/snippet}
  </PageHeader>

  {#if diskUsage.error && !snapshot}
    <p class="error">Couldn't read the storage devices: {diskUsage.error}</p>
  {:else if !snapshot}
    <div class="skeleton" style="height: 96px; margin-bottom: 16px"></div>
    <div class="skeleton" style="height: 320px"></div>
  {:else if overview}
    <StatTiles
      items={[
        { label: "Disks", value: String(overview.disks), sub: formatBytes(overview.capacity) },
        {
          label: "Used",
          value: formatBytes(overview.used),
          sub: `${usedShare.toFixed(0)}% of mounted`,
          tone: usedShare >= 95 ? "danger" : usedShare >= 85 ? "warn" : undefined,
        },
        { label: "Free", value: formatBytes(overview.available) },
        {
          label: "Not mounted",
          value: String(overview.unmounted),
          sub: overview.unmounted ? "mount to browse" : "all browsable",
          tone: overview.unmounted ? "warn" : undefined,
        },
      ]}
    />

    <p class="hint">
      Click a drive, partition or folder to open it. Sizes are the space taken on disk; right-click
      for more.
    </p>

    {#if snapshot.disks.length === 0}
      <p class="empty">No storage devices found.</p>
    {:else}
      <DiskTree
        {rows}
        ontoggle={(row) => diskUsage.toggle(row)}
        onmount={(volume) => diskUsage.mount(volume)}
        oncontext={(row, e) => {
          const items = treeMenu(row);
          if (items.length) menu.open(e, items, menuTitle(row));
        }}
      />
    {/if}
  {/if}
</Page>

<style>
  .check {
    display: inline-flex;
    align-items: center;
    gap: var(--s-2);
    color: var(--text-2);
    font-size: var(--fs-small);
    cursor: pointer;
  }
  .hint {
    margin-bottom: var(--s-3);
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
