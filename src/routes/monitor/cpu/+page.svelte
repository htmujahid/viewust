<script lang="ts">
  import MonitorCard from "$lib/features/monitor/MonitorCard.svelte";
  import MonitorGrid from "$lib/features/monitor/MonitorGrid.svelte";
  import Facts from "$lib/components/ui/Facts.svelte";
  import LineChart from "$lib/components/charts/LineChart.svelte";
  import Meter from "$lib/components/ui/Meter.svelte";
  import { monitor } from "$lib/features/monitor/store.svelte";
  import * as m from "$lib/features/monitor/series";

  const samples = $derived(monitor.samples);
  const now = $derived(monitor.latest)!;
  const tone = (load: number) => 0.08 + (Math.min(load, 100) / 100) * 0.92;

  const busiest = $derived(
    now.cpu.cores
      .map((load, i) => ({ load, i }))
      .sort((a, b) => b.load - a.load)
      .slice(0, 8),
  );
  const hasTemp = $derived(samples.some((s) => s.cpu.temperature !== null));
</script>

<MonitorGrid>
  <MonitorCard title="Processor load" wide>
    {#snippet value()}{m.percent(now.cpu.total)}
      <small>across {now.cpu.cores.length} threads</small>{/snippet}
    {#snippet aside()}
      <Facts
        items={[
          { label: "Clock", value: m.mhz(now.cpu.freq_mhz) },
          {
            label: "Temperature",
            value: now.cpu.temperature !== null ? m.celsius(now.cpu.temperature) : null,
          },
          { label: "Load average", value: now.cpu.load.map((l) => l.toFixed(2)).join(" · ") },
        ]}
      />
    {/snippet}

    <LineChart
      series={[{ name: "CPU load", color: m.COLOR_1, values: m.cpuLoad(samples) }]}
      format={m.percent}
      max={100}
      height={220}
      label="Processor load, last 60 seconds"
    />
  </MonitorCard>

  <MonitorCard title="Clock speed" wide={!hasTemp}>
    {#snippet value()}{m.mhz(now.cpu.freq_mhz)} <small>average across threads</small>{/snippet}

    <LineChart
      series={[{ name: "Clock", color: m.COLOR_1, values: m.cpuClock(samples) }]}
      format={m.mhz}
      height={150}
      label="Processor clock speed, last 60 seconds"
    />
  </MonitorCard>

  {#if hasTemp}
    <MonitorCard title="Temperature">
      {#snippet value()}{now.cpu.temperature !== null ? m.celsius(now.cpu.temperature) : "—"}
        <small>package</small>{/snippet}

      <LineChart
        series={[{ name: "Temperature", color: m.COLOR_1, values: m.cpuTemp(samples) }]}
        format={m.celsius}
        max={100}
        height={150}
        label="Processor temperature, last 60 seconds"
      />
    </MonitorCard>
  {/if}
  <MonitorCard title="Every thread, right now" wide>
    <div class="threads">
      <div class="heat-side">
        <div class="heat" role="list">
          {#each now.cpu.cores as load, i}
            <div
              class="cell"
              role="listitem"
              title="Thread {i}: {load.toFixed(0)}%"
              aria-label="Thread {i}: {load.toFixed(0)}%"
            >
              <span style:opacity={tone(load)}></span>
              <b>{i}</b>
            </div>
          {/each}
        </div>
        <p class="m-muted scale">
          Lighter means idle, deeper blue means busy. Hover a square for its exact load.
        </p>
      </div>

      <div class="busy-side">
        <h3 class="m-title sub">Busiest threads</h3>
        <ul class="m-rows">
          {#each busiest as t (t.i)}
            <li>
              <div class="m-line">
                <span>Thread {t.i}</span><b class="tabular">{m.percent(t.load)}</b>
              </div>
              <Meter value={t.load} tone="accent" height={4} />
            </li>
          {/each}
        </ul>
      </div>
    </div>
  </MonitorCard>
</MonitorGrid>

<style>
  .threads {
    display: grid;
    gap: var(--s-4);
  }
  .busy-side .sub {
    margin: 0 0 var(--s-2);
  }
  .busy-side :global(.m-rows) {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(220px, 1fr));
    column-gap: var(--s-5);
    margin-top: 0;
  }
  .heat {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(44px, 1fr));
    gap: 4px;
  }
  .cell {
    position: relative;
    aspect-ratio: 1;
    border-radius: 0;
    background: var(--surface-2);
    overflow: hidden;
  }
  .cell span {
    position: absolute;
    inset: 0;
    background: var(--series-1);
    transition: opacity 0.4s;
  }
  .cell b {
    position: absolute;
    left: 5px;
    bottom: 3px;
    color: var(--text);
    font-size: 10px;
    font-weight: 600;
    mix-blend-mode: normal;
    text-shadow: 0 0 3px var(--surface);
  }
  .scale {
    margin-top: var(--s-2);
  }
  .sub {
    margin: var(--s-4) 0 var(--s-1);
  }
</style>
