<script lang="ts">
  import MonitorCard from "$lib/features/monitor/MonitorCard.svelte";
  import MonitorGrid from "$lib/features/monitor/MonitorGrid.svelte";
  import LineChart from "$lib/components/charts/LineChart.svelte";
  import { monitor } from "$lib/features/monitor/store.svelte";
  import * as m from "$lib/features/monitor/series";
  import { groups, hasPower, hasThermal, hottestSeries } from "$lib/features/monitor/thermal";
  import { formatBytes, formatRate } from "$lib/utils/format";

  const samples = $derived(monitor.samples);
  const now = $derived(monitor.latest)!;

  const thermal = $derived(hasThermal(now));
  const powered = $derived(hasPower(now));
  const hottest = $derived(
    groups(now).reduce<ReturnType<typeof groups>[number] | null>(
      (a, b) => (!a || b.hottest.celsius > a.hottest.celsius ? b : a),
      null,
    ),
  );
  const drawn = $derived(
    (now.power.cpu_watts ?? 0) + now.gpus.reduce((n, g) => n + (g.power ?? 0), 0),
  );
  const drawSeries = $derived([
    ...(now.power.cpu_readable
      ? [{ name: "Processor", color: m.COLOR_1, values: m.cpuWatts(samples) }]
      : []),
    ...now.gpus
      .filter((g) => g.power !== null)
      .map((g, i) => ({
        name: g.name,
        color: m.colorAt(i + (now.power.cpu_readable ? 1 : 0)),
        values: m.gpuPower(samples, g.id),
      })),
  ]);
  const cardCount = $derived(4 + now.gpus.length + (thermal ? 1 : 0) + (powered ? 1 : 0));
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

    {#each now.gpus as g (g.id)}
      <MonitorCard title={`Graphics · ${g.name}`} href="/monitor/gpu">
        {#snippet value()}{g.util !== null
            ? m.percent(g.util)
            : g.core_mhz !== null
              ? m.mhz(g.core_mhz)
              : "—"}
          <small>{g.util !== null ? "load" : g.core_mhz !== null ? "clock" : "no readings"}</small
          >{/snippet}

        <LineChart
          series={g.util !== null || g.core_mhz === null
            ? [{ name: "GPU load", color: m.COLOR_1, values: m.gpuUtil(samples, g.id) }]
            : [{ name: "Core clock", color: m.COLOR_1, values: m.gpuClock(samples, g.id) }]}
          format={g.util !== null || g.core_mhz === null ? m.percent : m.mhz}
          max={g.util !== null || g.core_mhz === null ? 100 : undefined}
          height={110}
          table={false}
          label="{g.name}, last 60 seconds"
        />
      </MonitorCard>
    {/each}

    {#if thermal && hottest}
      <MonitorCard title="Thermal" href="/monitor/thermal">
        {#snippet value()}{m.celsius(hottest.hottest.celsius)}
          <small>hottest · {hottest.name}</small>{/snippet}

        <LineChart
          series={[{ name: "Hottest sensor", color: m.COLOR_1, values: hottestSeries(samples) }]}
          format={m.celsius}
          max={100}
          height={110}
          table={false}
          label="Hottest temperature, last 60 seconds"
        />
      </MonitorCard>
    {/if}

    {#if powered}
      <MonitorCard title="Power" href="/monitor/power">
        {#snippet value()}{now.power.batteries.length && !drawSeries.length
            ? `${now.power.batteries[0].percent.toFixed(0)}%`
            : m.watts(drawn)}
          <small
            >{now.power.batteries.length && !drawSeries.length
              ? "battery"
              : now.power.cpu_readable
                ? "processor + graphics"
                : "graphics"}</small
          >{/snippet}

        {#if drawSeries.length}
          <LineChart
            series={drawSeries}
            format={m.watts}
            floor={20}
            height={110}
            table={false}
            label="Power draw, last 60 seconds"
          />
        {:else}
          <LineChart
            series={[
              {
                name: "Charge",
                color: m.COLOR_1,
                values: m.batteryPercent(samples, now.power.batteries[0]?.name ?? ""),
              },
            ]}
            format={m.percent}
            max={100}
            height={110}
            table={false}
            label="Battery charge, last 60 seconds"
          />
        {/if}
      </MonitorCard>
    {/if}

    <MonitorCard title="Network" href="/monitor/network" wide={cardCount % 2 === 1}>
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
