<script lang="ts">
  import Facts from "$lib/components/Facts.svelte";
  import LineChart from "$lib/components/LineChart.svelte";
  import Meter from "$lib/components/Meter.svelte";
  import { monitor } from "$lib/monitor.svelte";
  import * as m from "$lib/monitor-series";
  import { formatBytes, formatRate } from "$lib/system";

  const samples = $derived(monitor.samples);
  const now = $derived(monitor.latest)!;
</script>

<div class="m-grid">
  <section class="card m-card m-wide">
    <header class="m-head">
      <div>
        <h2 class="m-title">All drives</h2>
        <p class="m-big tabular">
          {formatRate(m.last(m.diskRead(samples)))} <small>read</small> · {formatRate(m.last(m.diskWrite(samples)))} <small>write</small>
        </p>
      </div>
      <Facts items={[{ label: "Drives", value: String(now.disks.length) }, { label: "Volumes", value: String(now.volumes.length) }]} />
    </header>
    <LineChart
      series={[
        { name: "Read", color: m.COLOR_1, values: m.diskRead(samples) },
        { name: "Write", color: m.COLOR_2, values: m.diskWrite(samples) },
      ]}
      format={formatRate}
      floor={1e6}
      height={200}
      label="Disk read and write speed, last 60 seconds"
    />
  </section>

  {#each now.disks as d (d.name)}
    <section class="card m-card">
      <header class="m-head">
        <div>
          <h2 class="m-title">{d.name}{d.model ? ` · ${d.model}` : ""}</h2>
          <p class="m-big tabular">{m.percent(d.busy)} <small>busy</small></p>
        </div>
        <Facts items={[{ label: "Read", value: formatRate(d.read_bps) }, { label: "Write", value: formatRate(d.write_bps) }]} />
      </header>
      <LineChart
        series={[
          { name: "Read", color: m.COLOR_1, values: m.diskRead(samples, d.name) },
          { name: "Write", color: m.COLOR_2, values: m.diskWrite(samples, d.name) },
        ]}
        format={formatRate}
      floor={1e6}
        height={130}
        table={false}
        label="{d.name} read and write speed, last 60 seconds"
      />
      <div class="busy">
        <span class="m-muted">Time spent busy</span>
        <Meter value={d.busy} tone="accent" height={6} />
      </div>
    </section>
  {/each}

  <section class="card m-card m-wide">
    <header class="m-head"><h2 class="m-title">Space</h2></header>
    <ul class="m-rows">
      {#each now.volumes as v (v.mount)}
        <li>
          <div class="m-line">
            <span><b>{v.mount}</b> <span class="m-muted">{v.file_system}</span></span>
            <span class="m-muted tabular">{formatBytes(v.used)} used of {formatBytes(v.total)} · {formatBytes(v.total - v.used)} free</span>
          </div>
          <Meter value={(v.used / v.total) * 100} height={6} />
        </li>
      {/each}
    </ul>
  </section>
</div>

<style>
  .busy {
    margin-top: var(--s-3);
    display: flex;
    flex-direction: column;
    gap: 4px;
  }
</style>
