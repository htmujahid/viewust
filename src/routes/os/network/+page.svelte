<script lang="ts">
  import { onMount } from "svelte";

  import { goto } from "$app/navigation";

  import type { NetInterface } from "$lib/api/types";
  import Badge from "$lib/components/ui/Badge.svelte";
  import DataTable, { type Column } from "$lib/components/ui/DataTable.svelte";
  import DetailList from "$lib/components/ui/DetailList.svelte";
  import Masonry from "$lib/components/ui/Masonry.svelte";
  import RescanButton from "$lib/components/ui/RescanButton.svelte";
  import SectionShell from "$lib/components/ui/SectionShell.svelte";
  import StatTiles from "$lib/components/ui/StatTiles.svelte";
  import { osOverview } from "$lib/features/os/overview.svelte";
  import { menu } from "$lib/stores/menu.svelte";
  import { copyText } from "$lib/utils/actions";
  import { groupBySection } from "$lib/utils/details";
  import { formatBytes } from "$lib/utils/format";
  import { sectionCost } from "$lib/utils/masonry";

  // A stable handle: the Loaded instance itself never changes, only its fields do.
  const net = osOverview.network;
  const data = $derived(net.data);
  const items = $derived(
    groupBySection(data?.details ?? []).map((section) => ({
      key: section.title,
      cost: sectionCost(section.rows),
      section,
    })),
  );

  const KIND: Record<string, string> = {
    ethernet: "Ethernet",
    wifi: "Wi-Fi",
    bridge: "Bridge",
    virtual: "Virtual",
    loopback: "Loopback",
  };

  const columns: Column[] = [
    { key: "name", label: "Interface", sortable: false },
    { key: "state", label: "State", sortable: false },
    { key: "ipv4", label: "IPv4", sortable: false },
    { key: "mac", label: "MAC", sortable: false },
    { key: "link", label: "Link", sortable: false, hint: "Negotiated speed and packet size" },
    {
      key: "traffic",
      label: "Traffic",
      right: true,
      sortable: false,
      hint: "Received / sent since boot",
    },
  ];

  const stateTone = (i: NetInterface) =>
    i.state === "UP"
      ? ("ok" as const)
      : i.state === "DOWN"
        ? ("neutral" as const)
        : ("neutral" as const);

  function onkeydown(e: KeyboardEvent) {
    if (e.key === "Escape") goto("/os");
  }

  onMount(net.ensure);
</script>

<svelte:window {onkeydown} />

<SectionShell scroll title="Networking">
  {#snippet actions()}
    <RescanButton loading={net.loading} onclick={net.load} label />
  {/snippet}

  {#if net.error && !data}
    <p class="error">Couldn't read the network: {net.error}</p>
  {:else if !data}
    <div class="skeleton" style="height: 96px; margin-bottom: 16px"></div>
    <div class="skeleton" style="height: 320px"></div>
  {:else}
    <StatTiles
      items={[
        {
          label: "Interfaces up",
          value: `${data.up} of ${data.total}`,
          tone: data.up === 0 ? "danger" : undefined,
        },
        {
          label: "Internet",
          value: data.default_route ? "Reachable" : "No route",
          sub: data.default_route ?? undefined,
          tone: data.default_route ? "ok" : "warn",
        },
        { label: "Listening", value: String(data.listening_tcp.length), sub: "TCP ports" },
        { label: "Connections", value: String(data.established), sub: "established" },
      ]}
    />

    <div class="card table">
      <DataTable
        rows={data.interfaces}
        {columns}
        rowKey={(i) => i.name}
        sortKey=""
        sortDesc={false}
        emptyText="No network interfaces."
        onsort={() => {}}
        onselect={() => {}}
        oncontext={(i, e) =>
          menu.open(
            e,
            [
              { label: "Copy name", onselect: () => copyText(i.name, "interface name") },
              ...(i.ipv4.length
                ? [
                    {
                      label: "Copy IPv4",
                      hint: i.ipv4[0],
                      onselect: () => copyText(i.ipv4[0].split("/")[0], "address"),
                    },
                  ]
                : []),
              ...(i.mac
                ? [{ label: "Copy MAC", onselect: () => copyText(i.mac!, "MAC address") }]
                : []),
            ],
            i.name,
          )}
      >
        {#snippet cell(i, c)}
          {#if c.key === "name"}
            <b>{i.name}</b>
            <small class="muted">{KIND[i.kind] ?? i.kind}</small>
          {:else if c.key === "state"}
            <Badge tone={stateTone(i)}>{i.state === "UNKNOWN" ? "—" : i.state}</Badge>
          {:else if c.key === "ipv4"}
            {#if i.ipv4.length}
              <span class="tabular">{i.ipv4.join(", ")}</span>
              {#if i.ipv6}<small class="muted">+{i.ipv6} IPv6</small>{/if}
            {:else}
              <span class="muted">—</span>
            {/if}
          {:else if c.key === "mac"}
            <span class="tabular muted">{i.mac ?? "—"}</span>
          {:else if c.key === "link"}
            <span class="muted"
              >{[i.speed, i.mtu ? `MTU ${i.mtu}` : null].filter(Boolean).join(" · ") || "—"}</span
            >
          {:else}
            <span class="tabular muted">↓{formatBytes(i.rx)} ↑{formatBytes(i.tx)}</span>
          {/if}
        {/snippet}
      </DataTable>
    </div>

    <Masonry {items}>
      {#snippet children(item)}
        <section class="card block">
          <h2>{item.section.title}</h2>
          <DetailList rows={item.section.rows} stackAt={64} />
        </section>
      {/snippet}
    </Masonry>
  {/if}
</SectionShell>

<style>
  .table {
    margin: 0 0 var(--s-5);
    overflow-x: auto;
  }
  .table :global(.wrap) {
    overflow: visible;
    min-height: 0;
    border: 0;
  }
  .muted {
    color: var(--text-2);
  }
  small {
    display: block;
    font-family: var(--font-ui);
    font-size: var(--fs-label);
  }
  .block {
    margin: 0;
    padding: var(--s-4) var(--s-5);
  }
  .block h2 {
    margin-bottom: var(--s-3);
    color: var(--accent);
    font-family: var(--font-mono);
    font-size: var(--fs-label);
    font-weight: 700;
    letter-spacing: 0.06em;
    text-transform: uppercase;
  }
  .error {
    padding: var(--s-6) 0;
    color: var(--danger);
    text-align: center;
  }
</style>
