<script lang="ts">
  import LineChart from "$lib/components/charts/LineChart.svelte";
  import Facts from "$lib/components/ui/Facts.svelte";
  import MonitorCard from "$lib/features/monitor/MonitorCard.svelte";
  import MonitorGrid from "$lib/features/monitor/MonitorGrid.svelte";
  import * as m from "$lib/features/monitor/series";
  import { monitor } from "$lib/features/monitor/store.svelte";
  import { formatBytes } from "$lib/utils/format";

  const samples = $derived(monitor.samples);
  const now = $derived(monitor.latest)!;
</script>

{#each now.gpus as g, i (g.name + i)}
  {@const extras = [g.temperature, g.power, g.core_mhz].filter((v) => v !== null).length}
  <div class="card-group">
    <MonitorGrid>
      <MonitorCard title={g.name} wide>
        {#snippet value()}{g.util !== null ? m.percent(g.util) : "—"} <small>load</small>{/snippet}
        {#snippet aside()}
          <Facts
            items={[
              {
                label: "Video memory",
                value:
                  g.memory_used !== null && g.memory_total
                    ? `${formatBytes(g.memory_used)} of ${formatBytes(g.memory_total)}`
                    : null,
              },
              {
                label: "Temperature",
                value: g.temperature !== null ? m.celsius(g.temperature) : null,
              },
              {
                label: "Power",
                value:
                  g.power !== null
                    ? `${m.watts(g.power)}${g.power_limit ? ` of ${m.watts(g.power_limit)}` : ""}`
                    : null,
              },
              { label: "Fan", value: g.fan !== null ? `${g.fan.toFixed(0)}%` : null },
            ]}
          />
        {/snippet}
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
      </MonitorCard>

      {#if g.temperature !== null}
        {@const temperature = g.temperature}
        <MonitorCard title="Temperature" wide={extras === 1}>
          {#snippet value()}{m.celsius(temperature)}{/snippet}
          <LineChart
            series={[{ name: "Temperature", color: m.COLOR_1, values: m.gpuTemp(samples, i) }]}
            format={m.celsius}
            max={100}
            height={140}
            label="Graphics card temperature, last 60 seconds"
          />
        </MonitorCard>
      {/if}

      {#if g.power !== null}
        {@const power = g.power}
        {@const limit = g.power_limit}
        <MonitorCard title="Power draw" wide={extras % 2 === 1 && g.core_mhz === null}>
          {#snippet value()}{m.watts(power)}{#if limit}
              <small>of {m.watts(limit)} limit</small>{/if}{/snippet}
          <LineChart
            series={[{ name: "Power", color: m.COLOR_1, values: m.gpuPower(samples, i) }]}
            format={m.watts}
            max={limit ?? undefined}
            height={140}
            label="Graphics card power draw, last 60 seconds"
          />
        </MonitorCard>
      {/if}

      {#if g.core_mhz !== null}
        {@const core = g.core_mhz}
        {@const memory = g.memory_mhz}
        <MonitorCard title="Core clock" wide={extras % 2 === 1}>
          {#snippet value()}{m.mhz(core)}{#if memory}<small>
                · memory {m.mhz(memory)}</small
              >{/if}{/snippet}
          <LineChart
            series={[{ name: "Core clock", color: m.COLOR_1, values: m.gpuClock(samples, i) }]}
            format={m.mhz}
            height={140}
            label="Graphics card clock speed, last 60 seconds"
          />
        </MonitorCard>
      {/if}
    </MonitorGrid>
  </div>
{/each}

<style>
  .card-group + .card-group {
    margin-top: var(--s-5);
  }
</style>
