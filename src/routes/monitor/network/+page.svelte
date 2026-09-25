<script lang="ts">
  import { goto } from "$app/navigation";
  import Facts from "$lib/components/Facts.svelte";
  import Icon from "$lib/components/Icon.svelte";
  import LineChart from "$lib/components/LineChart.svelte";
  import { monitor } from "$lib/monitor.svelte";
  import * as m from "$lib/monitor-series";
  import { formatBytes, formatRate } from "$lib/system";

  const samples = $derived(monitor.samples);
  const now = $derived(monitor.latest)!;
  const speed = (mbps: number | null) => (mbps ? (mbps >= 1000 ? `${mbps / 1000} Gbit/s` : `${mbps} Mbit/s`) : null);
</script>

<div class="m-grid">
  <section class="card m-card m-wide">
    <header class="m-head">
      <div>
        <h2 class="m-title">All network traffic</h2>
        <p class="m-big tabular">
          {formatRate(m.last(m.netDown(samples)))} <small>down</small> · {formatRate(m.last(m.netUp(samples)))} <small>up</small>
        </p>
      </div>
      <Facts items={[{ label: "Active adapters", value: String(now.net.length) }]} />
    </header>
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
  </section>

  {#each now.net as n (n.name)}
    <section class="card m-card">
      <header class="m-head">
        <div>
          <h2 class="m-title">
            <Icon name={n.kind === "wifi" ? "wireless" : "network"} size={13} />
            {n.name} · {n.kind === "wifi" ? "Wi-Fi" : "Ethernet"}{n.default_route ? " · your internet connection" : ""}
          </h2>
          <p class="m-big tabular">{formatRate(n.rx_bps)} <small>down</small> · {formatRate(n.tx_bps)} <small>up</small></p>
        </div>
        <Facts
          items={[
            { label: "Link speed", value: speed(n.speed_mbps) },
            { label: "Received", value: formatBytes(n.rx_total) },
            { label: "Sent", value: formatBytes(n.tx_total) },
          ]}
        />
      </header>
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
    </section>
  {:else}
    <section class="card m-card m-wide"><p class="m-muted">No active network adapters.</p></section>
  {/each}

  <p class="m-muted m-wide">
    See how the connection reaches the internet, with the router and signal strength:
    <button class="m-link plain" onclick={() => goto("/")}>Open the devices map →</button>
  </p>
</div>

<style>
  .m-title {
    display: inline-flex;
    align-items: center;
    gap: 6px;
  }
  .plain {
    padding: 0;
    border: 0;
    background: transparent;
    cursor: pointer;
  }
</style>
