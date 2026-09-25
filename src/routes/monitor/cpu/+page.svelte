<script lang="ts">
  import Facts from "$lib/components/Facts.svelte";
  import LineChart from "$lib/components/LineChart.svelte";
  import Meter from "$lib/components/Meter.svelte";
  import { monitor } from "$lib/monitor.svelte";
  import * as m from "$lib/monitor-series";

  const samples = $derived(monitor.samples);
  const now = $derived(monitor.latest)!;
  const tone = (load: number) => 0.08 + (Math.min(load, 100) / 100) * 0.92;

  // threads ranked by how busy they are right now
  const busiest = $derived(
    now.cpu.cores.map((load, i) => ({ load, i })).sort((a, b) => b.load - a.load).slice(0, 8),
  );
  const hasTemp = $derived(samples.some((s) => s.cpu.temperature !== null));
</script>

<div class="m-grid">
  <section class="card m-card m-wide">
    <header class="m-head">
      <div>
        <h2 class="m-title">Processor load</h2>
        <p class="m-big tabular">{m.percent(now.cpu.total)} <small>across {now.cpu.cores.length} threads</small></p>
      </div>
      <Facts
        items={[
          { label: "Clock", value: m.mhz(now.cpu.freq_mhz) },
          { label: "Temperature", value: now.cpu.temperature !== null ? m.celsius(now.cpu.temperature) : null },
          { label: "Load average", value: now.cpu.load.map((l) => l.toFixed(2)).join(" · ") },
        ]}
      />
    </header>
    <LineChart series={[{ name: "CPU load", color: m.COLOR_1, values: m.cpuLoad(samples) }]} format={m.percent} max={100} height={220} label="Processor load, last 60 seconds" />
  </section>

  <section class="card m-card">
    <header class="m-head">
      <h2 class="m-title">Every thread, right now</h2>
    </header>
    <div class="heat" role="list">
      {#each now.cpu.cores as load, i}
        <div class="cell" role="listitem" title="Thread {i}: {load.toFixed(0)}%" aria-label="Thread {i}: {load.toFixed(0)}%">
          <span style:opacity={tone(load)}></span>
          <b>{i}</b>
        </div>
      {/each}
    </div>
    <p class="m-muted scale">Lighter means idle, deeper blue means busy. Hover a square for its exact load.</p>

    <h3 class="m-title sub">Busiest threads</h3>
    <ul class="m-rows">
      {#each busiest as t (t.i)}
        <li>
          <div class="m-line"><span>Thread {t.i}</span><b class="tabular">{m.percent(t.load)}</b></div>
          <Meter value={t.load} tone="accent" height={4} />
        </li>
      {/each}
    </ul>
  </section>

  <section class="card m-card">
    <header class="m-head">
      <div>
        <h2 class="m-title">Clock speed</h2>
        <p class="m-big tabular">{m.mhz(now.cpu.freq_mhz)} <small>average across threads</small></p>
      </div>
    </header>
    <LineChart series={[{ name: "Clock", color: m.COLOR_1, values: m.cpuClock(samples) }]} format={m.mhz} height={150} label="Processor clock speed, last 60 seconds" />
  </section>

  {#if hasTemp}
    <section class="card m-card">
      <header class="m-head">
        <div>
          <h2 class="m-title">Temperature</h2>
          <p class="m-big tabular">{now.cpu.temperature !== null ? m.celsius(now.cpu.temperature) : "—"} <small>package</small></p>
        </div>
      </header>
      <LineChart series={[{ name: "Temperature", color: m.COLOR_1, values: m.cpuTemp(samples) }]} format={m.celsius} max={100} height={150} label="Processor temperature, last 60 seconds" />
    </section>
  {/if}

  <p class="m-muted m-wide">
    Looking for which program is using the processor?
    <a class="m-link" href="/processes">Open processes →</a>
  </p>
</div>

<style>
  .heat {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(44px, 1fr));
    gap: 4px;
  }
  .cell {
    position: relative;
    aspect-ratio: 1;
    border-radius: 6px;
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
