<script lang="ts">
  import type { GpuSample } from "$lib/api/types";
  import LineChart from "$lib/components/charts/LineChart.svelte";
  import Badge from "$lib/components/ui/Badge.svelte";
  import Facts from "$lib/components/ui/Facts.svelte";
  import MonitorCard from "$lib/features/monitor/MonitorCard.svelte";
  import MonitorGrid from "$lib/features/monitor/MonitorGrid.svelte";
  import * as m from "$lib/features/monitor/series";
  import { monitor } from "$lib/features/monitor/store.svelte";
  import { formatBytes } from "$lib/utils/format";

  type Extra = "temp" | "power" | "clock" | "fan";

  const samples = $derived(monitor.samples);
  const now = $derived(monitor.latest)!;
  const gpus = $derived(now.gpus);

  const hasLoad = (g: GpuSample) => g.util !== null || !!g.memory_total;
  const extrasOf = (g: GpuSample): Extra[] =>
    (
      [
        g.temperature !== null && "temp",
        g.power !== null && "power",
        g.core_mhz !== null && !hasLoad(g) ? null : g.core_mhz !== null && "clock",
        g.fan !== null && "fan",
      ] as (Extra | false | null)[]
    ).filter((x): x is Extra => !!x);
  const isWide = (list: Extra[], key: Extra) => list.length % 2 === 1 && list.at(-1) === key;
  const empty = (g: GpuSample) => !hasLoad(g) && extrasOf(g).length === 0 && g.core_mhz === null;

  const withUtil = $derived(gpus.filter((g) => g.util !== null));
  const vramTotal = $derived(gpus.reduce((n, g) => n + (g.memory_total ?? 0), 0));
  const vramUsed = $derived(gpus.reduce((n, g) => n + (g.memory_used ?? 0), 0));
  const powerTotal = $derived(gpus.reduce((n, g) => n + (g.power ?? 0), 0));
  const anyPower = $derived(gpus.some((g) => g.power !== null));
</script>

{#if gpus.length > 1}
  <MonitorGrid>
    <MonitorCard title="All graphics" wide>
      {#snippet value()}{gpus.length} <small>graphics processors</small>{/snippet}
      {#snippet aside()}
        <Facts
          items={[
            {
              label: "Video memory",
              value: vramTotal ? `${formatBytes(vramUsed)} of ${formatBytes(vramTotal)}` : null,
            },
            { label: "Power", value: anyPower ? m.watts(powerTotal) : null },
          ]}
        />
      {/snippet}
      {#if withUtil.length}
        <LineChart
          series={withUtil.map((g, i) => ({
            name: g.name,
            color: m.colorAt(i),
            values: m.gpuUtil(samples, g.id),
          }))}
          format={m.percent}
          max={100}
          height={200}
          label="Load of every graphics processor, last 60 seconds"
        />
      {:else}
        <LineChart
          series={gpus.map((g, i) => ({
            name: g.name,
            color: m.colorAt(i),
            values: m.gpuClock(samples, g.id),
          }))}
          format={m.mhz}
          height={200}
          label="Clock speed of every graphics processor, last 60 seconds"
        />
      {/if}
    </MonitorCard>
  </MonitorGrid>
{/if}

{#each gpus as g (g.id)}
  {@const extras = extrasOf(g)}
  <section class="gpu">
    <header class="head">
      <h2>{g.name}</h2>
      <div class="tags">
        {#if g.kind !== "unknown"}<Badge tone={g.kind === "discrete" ? "ok" : "neutral"}
            >{g.kind}</Badge
          >{/if}
        <Badge>{g.vendor}</Badge>
        <span class="slot tabular">PCI {g.id}</span>
      </div>
    </header>

    <MonitorGrid>
      {#if empty(g)}
        <MonitorCard title="Live readings" wide>
          <p class="m-muted">
            The driver for this graphics processor doesn't report load, temperature or power without
            extra tools, so there is nothing to chart yet.
          </p>
        </MonitorCard>
      {:else if hasLoad(g)}
        <MonitorCard title={g.util !== null ? "Load and video memory" : "Video memory"} wide>
          {#snippet value()}{g.util !== null
              ? m.percent(g.util)
              : g.memory_used !== null && g.memory_total
                ? formatBytes(g.memory_used)
                : "—"}
            <small>{g.util !== null ? "load" : "in use"}</small>{/snippet}
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
              ...(g.util !== null
                ? [{ name: "GPU load", color: m.COLOR_1, values: m.gpuUtil(samples, g.id) }]
                : []),
              ...(g.memory_total
                ? [{ name: "Video memory", color: m.COLOR_2, values: m.gpuMem(samples, g.id) }]
                : []),
            ]}
            format={m.percent}
            max={100}
            height={200}
            label="{g.name} load and video memory, last 60 seconds"
          />
        </MonitorCard>
      {:else if g.core_mhz !== null}
        {@const core = g.core_mhz}
        <MonitorCard title="Clock speed" wide>
          {#snippet value()}{m.mhz(core)} <small>current</small>{/snippet}
          <LineChart
            series={[{ name: "Core clock", color: m.COLOR_1, values: m.gpuClock(samples, g.id) }]}
            format={m.mhz}
            height={180}
            label="{g.name} clock speed, last 60 seconds"
          />
        </MonitorCard>
      {/if}

      {#if g.temperature !== null}
        {@const temperature = g.temperature}
        <MonitorCard title="Temperature" wide={isWide(extras, "temp")}>
          {#snippet value()}{m.celsius(temperature)}{/snippet}
          <LineChart
            series={[{ name: "Temperature", color: m.COLOR_1, values: m.gpuTemp(samples, g.id) }]}
            format={m.celsius}
            max={100}
            height={140}
            label="{g.name} temperature, last 60 seconds"
          />
        </MonitorCard>
      {/if}

      {#if g.power !== null}
        {@const power = g.power}
        {@const limit = g.power_limit}
        <MonitorCard title="Power draw" wide={isWide(extras, "power")}>
          {#snippet value()}{m.watts(power)}{#if limit}
              <small>of {m.watts(limit)} limit</small>{/if}{/snippet}
          <LineChart
            series={[{ name: "Power", color: m.COLOR_1, values: m.gpuPower(samples, g.id) }]}
            format={m.watts}
            max={limit ?? undefined}
            height={140}
            label="{g.name} power draw, last 60 seconds"
          />
        </MonitorCard>
      {/if}

      {#if extras.includes("clock") && g.core_mhz !== null}
        {@const core = g.core_mhz}
        {@const memory = g.memory_mhz}
        <MonitorCard title="Core clock" wide={isWide(extras, "clock")}>
          {#snippet value()}{m.mhz(core)}{#if memory}<small>
                · memory {m.mhz(memory)}</small
              >{/if}{/snippet}
          <LineChart
            series={[{ name: "Core clock", color: m.COLOR_1, values: m.gpuClock(samples, g.id) }]}
            format={m.mhz}
            height={140}
            label="{g.name} clock speed, last 60 seconds"
          />
        </MonitorCard>
      {/if}

      {#if g.fan !== null}
        {@const fan = g.fan}
        <MonitorCard title="Fan" wide={isWide(extras, "fan")}>
          {#snippet value()}{fan.toFixed(0)}%{/snippet}
          <LineChart
            series={[{ name: "Fan speed", color: m.COLOR_1, values: m.gpuFan(samples, g.id) }]}
            format={m.percent}
            max={100}
            height={140}
            label="{g.name} fan speed, last 60 seconds"
          />
        </MonitorCard>
      {/if}
    </MonitorGrid>
  </section>
{/each}

<style>
  .gpu {
    margin-top: var(--s-5);
  }
  .gpu:first-child {
    margin-top: 0;
  }
  .head {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: var(--s-2) var(--s-4);
    margin-bottom: var(--s-4);
    padding-bottom: var(--s-2);
    border-bottom: 1px solid var(--border);
  }
  h2 {
    font-size: 15px;
    letter-spacing: 0.04em;
    overflow-wrap: anywhere;
  }
  h2::before {
    content: "> ";
    color: var(--accent);
  }
  .tags {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: var(--s-2);
  }
  .slot {
    color: var(--text-3);
    font-size: 11px;
  }
</style>
