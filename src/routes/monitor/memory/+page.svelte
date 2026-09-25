<script lang="ts">
  import Facts from "$lib/components/Facts.svelte";
  import LineChart from "$lib/components/LineChart.svelte";
  import { monitor } from "$lib/monitor.svelte";
  import * as m from "$lib/monitor-series";
  import { formatBytes } from "$lib/system";

  const samples = $derived(monitor.samples);
  const now = $derived(monitor.latest)!;
  const mem = $derived(now.memory);

  // Where the installed memory is right now. Cached memory counts as "available":
  // the system hands it back the moment a program asks.
  const free = $derived(Math.max(mem.total - mem.used - mem.cached, 0));
  const parts = $derived([
    { label: "In use by programs", value: mem.used, color: "var(--series-1)" },
    { label: "Cached files (reclaimable)", value: mem.cached, color: "color-mix(in srgb, var(--series-1) 35%, var(--surface-2))" },
    { label: "Free", value: free, color: "var(--surface-2)" },
  ]);
  const hasSwap = $derived(mem.swap_total > 0);
</script>

<div class="m-grid">
  <section class="card m-card m-wide">
    <header class="m-head">
      <div>
        <h2 class="m-title">Memory use</h2>
        <p class="m-big tabular">{formatBytes(mem.used)} <small>of {formatBytes(mem.total)} installed</small></p>
      </div>
      <Facts
        items={[
          { label: "Available", value: formatBytes(mem.available) },
          { label: "Cached", value: formatBytes(mem.cached) },
          { label: "Used", value: m.percent((mem.used / mem.total) * 100) },
        ]}
      />
    </header>
    <LineChart
      series={[
        { name: "Memory used", color: m.COLOR_1, values: m.memUsed(samples) },
        ...(hasSwap ? [{ name: "Swap used", color: m.COLOR_2, values: m.swapUsed(samples) }] : []),
      ]}
      format={m.percent}
      max={100}
      height={220}
      label="Memory and swap use, last 60 seconds"
    />
  </section>

  <section class="card m-card">
    <header class="m-head"><h2 class="m-title">Where it is right now</h2></header>
    <div class="bar" role="img" aria-label="Memory composition">
      {#each parts as p}
        <span style:width="{(p.value / mem.total) * 100}%" style:background={p.color} title="{p.label}: {formatBytes(p.value)}"></span>
      {/each}
    </div>
    <ul class="m-rows legend">
      {#each parts as p}
        <li>
          <div class="m-line">
            <span class="key"><i style:background={p.color}></i>{p.label}</span>
            <b class="tabular">{formatBytes(p.value)}</b>
          </div>
        </li>
      {/each}
    </ul>
    <p class="m-muted">
      Linux keeps recently used files in spare memory to speed things up and gives it back
      the instant a program needs it, so a high “used” figure is rarely a problem on its own.
    </p>
  </section>

  <section class="card m-card">
    <header class="m-head">
      <div>
        <h2 class="m-title">Swap</h2>
        <p class="m-big tabular">
          {#if hasSwap}{formatBytes(mem.swap_used)} <small>of {formatBytes(mem.swap_total)}</small>{:else}None{/if}
        </p>
      </div>
    </header>
    {#if hasSwap}
      <LineChart series={[{ name: "Swap used", color: m.COLOR_2, values: m.swapUsed(samples) }]} format={m.percent} max={100} height={150} label="Swap use, last 60 seconds" />
      <p class="m-muted">
        Swap is disk space used as overflow when memory is full. Occasional use is normal;
        constant growth means programs are asking for more than the computer has.
      </p>
    {:else}
      <p class="m-muted">This computer has no swap space configured.</p>
    {/if}
  </section>

  <p class="m-muted m-wide">
    Want to know which program is using it?
    <a class="m-link" href="/processes">Open processes, sorted by memory →</a>
  </p>
</div>

<style>
  .bar {
    display: flex;
    height: 14px;
    gap: 2px;
    border-radius: 999px;
    overflow: hidden;
  }
  .bar span {
    min-width: 2px;
    border-radius: 3px;
    transition: width 0.4s ease;
  }
  .legend {
    margin-bottom: var(--s-3);
  }
  .key {
    display: inline-flex;
    align-items: center;
    gap: var(--s-2);
  }
  .key i {
    width: 10px;
    height: 10px;
    border-radius: 3px;
  }
</style>
