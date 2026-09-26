<script lang="ts">
  import MonitorCard from "$lib/features/monitor/MonitorCard.svelte";
  import MonitorGrid from "$lib/features/monitor/MonitorGrid.svelte";
  import LineChart from "$lib/components/charts/LineChart.svelte";
  import { monitor } from "$lib/features/monitor/store.svelte";
  import * as m from "$lib/features/monitor/series";
  import { formatBytes, formatRate } from "$lib/utils/format";

  const samples = $derived(monitor.samples);
  const now = $derived(monitor.latest)!;
</script>

<!-- A glance at everything. Each card opens its own page with the full picture. -->
<MonitorGrid>
  <MonitorCard title="Processor" href="/monitor/cpu">
    {#snippet value()}{m.percent(now.cpu.total)}
      <small>across {now.cpu.cores.length} threads</small>{/snippet}

    <LineChart
      series={[{ name: "CPU load", color: m.COLOR_1, values: m.cpuLoad(samples) }]}
      format={m.percent}
      max={100}
      height={110}
      table={false}
      label="Processor load, last 60 seconds"
    />
  </MonitorCard>

  <MonitorCard title="Memory" href="/monitor/memory">
    {#snippet value()}{formatBytes(now.memory.used)}
      <small>of {formatBytes(now.memory.total)}</small>{/snippet}

    <LineChart
      series={[{ name: "Memory used", color: m.COLOR_1, values: m.memUsed(samples) }]}
      format={m.percent}
      max={100}
      height={110}
      table={false}
      label="Memory use, last 60 seconds"
    />
  </MonitorCard>

  <MonitorCard title="Storage" href="/monitor/storage">
    {#snippet value()}{formatRate(m.last(m.diskRead(samples)))} <small>read</small> · {formatRate(
        m.last(m.diskWrite(samples)),
      )} <small>write</small>{/snippet}

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
  </MonitorCard>

  {#each now.gpus as g, i (g.name + i)}
    <MonitorCard title={`Graphics · ${g.name}`} href="/monitor/gpu">
      {#snippet value()}{g.util !== null ? m.percent(g.util) : "—"} <small>load</small>{/snippet}

      <LineChart
        series={[{ name: "GPU load", color: m.COLOR_1, values: m.gpuUtil(samples, i) }]}
        format={m.percent}
        max={100}
        height={110}
        table={false}
        label="Graphics card load, last 60 seconds"
      />
    </MonitorCard>
  {/each}

  <MonitorCard title="Network" href="/monitor/network">
    {#snippet value()}{formatRate(m.last(m.netDown(samples)))} <small>down</small> · {formatRate(
        m.last(m.netUp(samples)),
      )} <small>up</small>{/snippet}

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
  </MonitorCard>
</MonitorGrid>
