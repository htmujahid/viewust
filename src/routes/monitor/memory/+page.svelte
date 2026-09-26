<script lang="ts">
  import MonitorCard from "$lib/features/monitor/MonitorCard.svelte";
  import MonitorGrid from "$lib/features/monitor/MonitorGrid.svelte";
  import Facts from "$lib/components/ui/Facts.svelte";
  import LineChart from "$lib/components/charts/LineChart.svelte";
  import { monitor } from "$lib/features/monitor/store.svelte";
  import * as m from "$lib/features/monitor/series";
  import { formatBytes } from "$lib/utils/format";

  const samples = $derived(monitor.samples);
  const now = $derived(monitor.latest)!;
  const mem = $derived(now.memory);

  // Where the installed memory is right now. Cached memory counts as "available":
  // the system hands it back the moment a program asks.
  const free = $derived(Math.max(mem.total - mem.used - mem.cached, 0));
  const parts = $derived([
    { label: "In use by programs", value: mem.used, color: "var(--series-1)" },
    {
      label: "Cached files (reclaimable)",
      value: mem.cached,
      color: "color-mix(in srgb, var(--series-1) 35%, var(--surface-2))",
    },
    { label: "Free", value: free, color: "var(--surface-2)" },
  ]);
  const hasSwap = $derived(mem.swap_total > 0);
</script>

<MonitorGrid>
  <MonitorCard title="Memory use" wide>
    {#snippet value()}{formatBytes(mem.used)}
      <small>of {formatBytes(mem.total)} installed</small>{/snippet}
    {#snippet aside()}
      <Facts
        items={[
          { label: "Available", value: formatBytes(mem.available) },
          { label: "Cached", value: formatBytes(mem.cached) },
          { label: "Used", value: m.percent((mem.used / mem.total) * 100) },
        ]}
      />
    {/snippet}

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
  </MonitorCard>

  <MonitorCard title="Where it is right now">
    <div class="bar" role="img" aria-label="Memory composition">
      {#each parts as p}
        <span
          style:width="{(p.value / mem.total) * 100}%"
          style:background={p.color}
          title="{p.label}: {formatBytes(p.value)}"
        ></span>
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
      Linux keeps recently used files in spare memory to speed things up and gives it back the
      instant a program needs it, so a high “used” figure is rarely a problem on its own.
    </p>
  </MonitorCard>

  <MonitorCard title="Swap">
    {#snippet value()}{#if hasSwap}{formatBytes(mem.swap_used)}
        <small>of {formatBytes(mem.swap_total)}</small>{:else}None{/if}{/snippet}

    {#if hasSwap}
      <LineChart
        series={[{ name: "Swap used", color: m.COLOR_2, values: m.swapUsed(samples) }]}
        format={m.percent}
        max={100}
        height={150}
        label="Swap use, last 60 seconds"
      />
      <p class="m-muted">
        Swap is disk space used as overflow when memory is full. Occasional use is normal; constant
        growth means programs are asking for more than the computer has.
      </p>
    {:else}
      <p class="m-muted">This computer has no swap space configured.</p>
    {/if}
  </MonitorCard>

  <p class="m-muted m-wide">
    Want to know which program is using it?
    <a class="m-link" href="/processes">Open processes, sorted by memory →</a>
  </p>
</MonitorGrid>

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
