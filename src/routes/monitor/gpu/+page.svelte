<script lang="ts">
  import Facts from "$lib/components/Facts.svelte";
  import LineChart from "$lib/components/LineChart.svelte";
  import { monitor } from "$lib/monitor.svelte";
  import * as m from "$lib/monitor-series";
  import { formatBytes } from "$lib/system";

  const samples = $derived(monitor.samples);
  const now = $derived(monitor.latest)!;
</script>

{#each now.gpus as g, i (g.name + i)}
  <div class="m-grid group">
    <section class="card m-card m-wide">
      <header class="m-head">
        <div>
          <h2 class="m-title">{g.name}</h2>
          <p class="m-big tabular">{g.util !== null ? m.percent(g.util) : "—"} <small>load</small></p>
        </div>
        <Facts
          items={[
            { label: "Video memory", value: g.memory_used !== null && g.memory_total ? `${formatBytes(g.memory_used)} of ${formatBytes(g.memory_total)}` : null },
            { label: "Temperature", value: g.temperature !== null ? m.celsius(g.temperature) : null },
            { label: "Power", value: g.power !== null ? `${m.watts(g.power)}${g.power_limit ? ` of ${m.watts(g.power_limit)}` : ""}` : null },
            { label: "Fan", value: g.fan !== null ? `${g.fan.toFixed(0)}%` : null },
          ]}
        />
      </header>
      <LineChart
        series={[
          { name: "GPU load", color: m.COLOR_1, values: m.gpuUtil(samples, i) },
          { name: "Video memory", color: m.COLOR_2, values: m.gpuMem(samples, i) },
        ]}
        format={m.percent}
        max={100}
        height={220}
        label="{g.name} load and video memory, last 60 seconds"
      />
    </section>

    {#if g.temperature !== null}
      <section class="card m-card">
        <header class="m-head">
          <div>
            <h2 class="m-title">Temperature</h2>
            <p class="m-big tabular">{m.celsius(g.temperature)}</p>
          </div>
        </header>
        <LineChart series={[{ name: "Temperature", color: m.COLOR_1, values: m.gpuTemp(samples, i) }]} format={m.celsius} max={100} height={140} label="Graphics card temperature, last 60 seconds" />
      </section>
    {/if}

    {#if g.power !== null}
      <section class="card m-card">
        <header class="m-head">
          <div>
            <h2 class="m-title">Power draw</h2>
            <p class="m-big tabular">{m.watts(g.power)} {#if g.power_limit}<small>of {m.watts(g.power_limit)} limit</small>{/if}</p>
          </div>
        </header>
        <LineChart series={[{ name: "Power", color: m.COLOR_1, values: m.gpuPower(samples, i) }]} format={m.watts} max={g.power_limit ?? undefined} height={140} label="Graphics card power draw, last 60 seconds" />
      </section>
    {/if}

    {#if g.core_mhz !== null}
      <section class="card m-card">
        <header class="m-head">
          <div>
            <h2 class="m-title">Core clock</h2>
            <p class="m-big tabular">{m.mhz(g.core_mhz)}{#if g.memory_mhz}<small> · memory {m.mhz(g.memory_mhz)}</small>{/if}</p>
          </div>
        </header>
        <LineChart series={[{ name: "Core clock", color: m.COLOR_1, values: m.gpuClock(samples, i) }]} format={m.mhz} height={140} label="Graphics card clock speed, last 60 seconds" />
      </section>
    {/if}
  </div>
{/each}

<style>
  .group + .group {
    margin-top: var(--s-5);
  }
</style>
