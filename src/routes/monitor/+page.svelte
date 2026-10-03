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

<div class="overview">
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
      {#snippet value()}<div class="metrics">
          <span>{formatRate(m.last(m.diskRead(samples)))}<small>Read / sec</small></span><span
            >{formatRate(m.last(m.diskWrite(samples)))}<small>Write / sec</small></span
          >
        </div>{/snippet}

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

    <MonitorCard title="Network" href="/monitor/network" wide={now.gpus.length % 2 === 1}>
      {#snippet value()}<div class="metrics">
          <span>{formatRate(m.last(m.netDown(samples)))}<small>Download / sec</small></span><span
            >{formatRate(m.last(m.netUp(samples)))}<small>Upload / sec</small></span
          >
        </div>{/snippet}

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
</div>

<style>
  .overview :global(section.card) {
    min-height: 280px;
  }
  .overview :global(section.card > header) {
    min-height: 36px;
    flex-wrap: nowrap;
    gap: 12px;
  }
  .overview :global(.value) {
    min-height: 64px;
    font-size: 26px;
  }
  .overview :global(.body) {
    margin-top: auto;
  }
  .overview :global(.chart) {
    padding-top: 28px;
    position: relative;
  }
  .overview :global(.chart > .legend) {
    position: absolute;
    top: 0;
    left: 0;
    margin: 0;
  }
  .metrics {
    display: grid;
    grid-template-columns: minmax(0, 1fr) minmax(0, 1fr);
    gap: 16px;
  }
  .metrics span {
    white-space: nowrap;
    font-size: clamp(18px, 1.6vw, 24px);
  }
  .metrics small {
    display: block;
    margin-top: 6px;
  }
</style>
