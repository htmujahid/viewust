<script lang="ts">
  import LineChart from "$lib/components/LineChart.svelte";
  import { monitor } from "$lib/monitor.svelte";
  import * as m from "$lib/monitor-series";
  import { formatBytes, formatRate } from "$lib/system";

  const samples = $derived(monitor.samples);
  const now = $derived(monitor.latest)!;
</script>

<!-- A glance at everything. Each card opens its own page with the full picture. -->
<div class="m-grid">
  <section class="card m-card">
    <header class="m-head">
      <div>
        <h2 class="m-title">Processor</h2>
        <p class="m-big tabular">{m.percent(now.cpu.total)} <small>across {now.cpu.cores.length} threads</small></p>
      </div>
      <a class="m-link" href="/monitor/cpu">Details →</a>
    </header>
    <LineChart
      series={[{ name: "CPU load", color: m.COLOR_1, values: m.cpuLoad(samples) }]}
      format={m.percent}
      max={100}
      height={110}
      table={false}
      label="Processor load, last 60 seconds"
    />
  </section>

  <section class="card m-card">
    <header class="m-head">
      <div>
        <h2 class="m-title">Memory</h2>
        <p class="m-big tabular">{formatBytes(now.memory.used)} <small>of {formatBytes(now.memory.total)}</small></p>
      </div>
      <a class="m-link" href="/monitor/memory">Details →</a>
    </header>
    <LineChart
      series={[{ name: "Memory used", color: m.COLOR_1, values: m.memUsed(samples) }]}
      format={m.percent}
      max={100}
      height={110}
      table={false}
      label="Memory use, last 60 seconds"
    />
  </section>

  <section class="card m-card">
    <header class="m-head">
      <div>
        <h2 class="m-title">Storage</h2>
        <p class="m-big tabular">
          {formatRate(m.last(m.diskRead(samples)))} <small>read</small> · {formatRate(m.last(m.diskWrite(samples)))} <small>write</small>
        </p>
      </div>
      <a class="m-link" href="/monitor/storage">Details →</a>
    </header>
    <LineChart
      series={[
        { name: "Read", color: m.COLOR_1, values: m.diskRead(samples) },
        { name: "Write", color: m.COLOR_2, values: m.diskWrite(samples) },
      ]}
      format={formatRate}
      floor={1e6}
      height={110}
      table={false}
      label="Disk speed, last 60 seconds"
    />
  </section>

  {#each now.gpus as g, i (g.name + i)}
    <section class="card m-card">
      <header class="m-head">
        <div>
          <h2 class="m-title">Graphics · {g.name}</h2>
          <p class="m-big tabular">{g.util !== null ? m.percent(g.util) : "—"} <small>load</small></p>
        </div>
        <a class="m-link" href="/monitor/gpu">Details →</a>
      </header>
      <LineChart
        series={[{ name: "GPU load", color: m.COLOR_1, values: m.gpuUtil(samples, i) }]}
        format={m.percent}
        max={100}
        height={110}
        table={false}
        label="Graphics card load, last 60 seconds"
      />
    </section>
  {/each}

  <section class="card m-card">
    <header class="m-head">
      <div>
        <h2 class="m-title">Network</h2>
        <p class="m-big tabular">
          {formatRate(m.last(m.netDown(samples)))} <small>down</small> · {formatRate(m.last(m.netUp(samples)))} <small>up</small>
        </p>
      </div>
      <a class="m-link" href="/monitor/network">Details →</a>
    </header>
    <LineChart
      series={[
        { name: "Download", color: m.COLOR_1, values: m.netDown(samples) },
        { name: "Upload", color: m.COLOR_2, values: m.netUp(samples) },
      ]}
      format={formatRate}
      floor={1e6}
      height={110}
      table={false}
      label="Network speed, last 60 seconds"
    />
  </section>
</div>
