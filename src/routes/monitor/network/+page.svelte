<script lang="ts">
  import MonitorCard from "$lib/features/monitor/MonitorCard.svelte";
  import MonitorGrid from "$lib/features/monitor/MonitorGrid.svelte";
  import { goto } from "$app/navigation";
  import Facts from "$lib/components/ui/Facts.svelte";
  import LineChart from "$lib/components/charts/LineChart.svelte";
  import { monitor } from "$lib/features/monitor/store.svelte";
  import * as m from "$lib/features/monitor/series";
  import { formatBytes, formatRate } from "$lib/utils/format";

  const samples = $derived(monitor.samples);
  const now = $derived(monitor.latest)!;
  const speed = (mbps: number | null) =>
    mbps ? (mbps >= 1000 ? `${mbps / 1000} Gbit/s` : `${mbps} Mbit/s`) : null;
</script>

<MonitorGrid>
  <MonitorCard title="All network traffic" wide>
    {#snippet value()}{formatRate(m.last(m.netDown(samples)))} <small>down</small> · {formatRate(
        m.last(m.netUp(samples)),
      )} <small>up</small>{/snippet}
    {#snippet aside()}
      <Facts items={[{ label: "Active adapters", value: String(now.net.length) }]} />
    {/snippet}

    <LineChart
      series={[
        { name: "Download", color: m.COLOR_1, values: m.netDown(samples) },
        { name: "Upload", color: m.COLOR_2, values: m.netUp(samples) },
      ]}
      format={formatRate}
      floor={1e6}
      height={220}
      label="Network download and upload speed, last 60 seconds"
    />
  </MonitorCard>

  {#each now.net as n (n.name)}
    <MonitorCard
      title={`${n.name} · ${n.kind === "wifi" ? "Wi-Fi" : "Ethernet"}${n.default_route ? " · your internet connection" : ""}`}
    >
      {#snippet value()}{formatRate(n.rx_bps)} <small>down</small> · {formatRate(n.tx_bps)}
        <small>up</small>{/snippet}
      {#snippet aside()}
        <Facts
          items={[
            { label: "Link speed", value: speed(n.speed_mbps) },
            { label: "Received", value: formatBytes(n.rx_total) },
            { label: "Sent", value: formatBytes(n.tx_total) },
          ]}
        />
      {/snippet}

      <LineChart
        series={[
          { name: "Download", color: m.COLOR_1, values: m.netDown(samples, n.name) },
          { name: "Upload", color: m.COLOR_2, values: m.netUp(samples, n.name) },
        ]}
        format={formatRate}
        floor={1e6}
        height={140}
        table={false}
        label="{n.name} download and upload speed, last 60 seconds"
      />
    </MonitorCard>
  {:else}
    <section class="card empty"><p class="m-muted">No active network adapters.</p></section>
  {/each}

  <p class="m-muted m-wide">
    See how the connection reaches the internet, with the router and signal strength:
    <button class="m-link plain" onclick={() => goto("/")}>Open the devices map →</button>
  </p>
</MonitorGrid>

<style>
  .empty {
    grid-column: 1 / -1;
    padding: var(--s-4);
  }
  .plain {
    padding: 0;
    border: 0;
    background: transparent;
    cursor: pointer;
  }
</style>
