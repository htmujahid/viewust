<script lang="ts">
  import LineChart from "$lib/components/charts/LineChart.svelte";
  import Facts from "$lib/components/ui/Facts.svelte";
  import Meter from "$lib/components/ui/Meter.svelte";
  import MonitorCard from "$lib/features/monitor/MonitorCard.svelte";
  import MonitorGrid from "$lib/features/monitor/MonitorGrid.svelte";
  import * as m from "$lib/features/monitor/series";
  import { monitor } from "$lib/features/monitor/store.svelte";
  import { groups, groupSeries, type Group, type Reading } from "$lib/features/monitor/thermal";

  const samples = $derived(monitor.samples);
  const now = $derived(monitor.latest)!;
  const all = $derived(groups(now));
  const fans = $derived(now.thermal.fans);
  const hottest = $derived(
    all.length ? all.reduce((a, b) => (b.hottest.celsius > a.hottest.celsius ? b : a)) : null,
  );
  const BIG = 3;
  const big = $derived(all.filter((g) => g.readings.length > BIG));
  const small = $derived(
    [
      ...all
        .filter((g) => g.readings.length <= BIG)
        .map((g) => ({ group: g, size: g.readings.length })),
      ...(fans.length ? [{ group: null, size: fans.length }] : []),
    ].sort((a, b) => b.size - a.size),
  );
  const lastSmallWide = (index: number) => small.length % 2 === 1 && index === small.length - 1;
  const colorOf = (name: string) => m.colorAt(all.findIndex((g) => g.name === name));

  function tone(r: Reading): "ok" | "warn" | "danger" {
    const near = (limit: number | null) => limit !== null && r.celsius >= limit - 5;
    if (near(r.critical)) return "danger";
    if (near(r.high)) return "warn";
    if (r.high === null && r.critical === null) {
      return r.celsius >= 85 ? "danger" : r.celsius >= 75 ? "warn" : "ok";
    }
    return "ok";
  }
  const scale = (r: Reading) => Math.min((r.celsius / (r.critical ?? r.high ?? 100)) * 100, 100);
  const limitText = (r: Reading) =>
    r.critical !== null
      ? `limit ${m.celsius(r.critical)}`
      : r.high !== null
        ? `warns at ${m.celsius(r.high)}`
        : "";
</script>

{#snippet group(g: Group, wide: boolean)}
  <MonitorCard title={g.name} {wide}>
    {#snippet value()}{m.celsius(g.hottest.celsius)}
      <small>{g.readings.length > 1 ? `hottest · ${g.hottest.label}` : g.hottest.label}</small
      >{/snippet}
    <LineChart
      series={[{ name: g.name, color: colorOf(g.name), values: groupSeries(samples, g.name) }]}
      format={m.celsius}
      max={100}
      height={110}
      table={false}
      label="{g.name} temperature, last 60 seconds"
    />
    <ul class="m-rows" class:cols={wide && g.readings.length > BIG}>
      {#each g.readings as r (r.id)}
        <li>
          <div class="m-line">
            <span>{r.label}</span>
            <b class="tabular">
              {m.celsius(r.celsius)}
              {#if limitText(r)}<span class="m-muted"> · {limitText(r)}</span>{/if}
            </b>
          </div>
          <Meter value={scale(r)} tone={tone(r)} height={4} />
        </li>
      {/each}
    </ul>
  </MonitorCard>
{/snippet}

{#snippet fansCard(wide: boolean)}
  <MonitorCard title="Fans" {wide}>
    {#snippet value()}{Math.round(Math.max(...fans.map((f) => f.rpm)))}
      <small>RPM, fastest</small>{/snippet}
    <LineChart
      series={fans.map((f, i) => ({
        name: f.label,
        color: m.colorAt(i),
        values: m.fanRpm(samples, f.id),
      }))}
      format={(v) => `${Math.round(v)} RPM`}
      floor={1000}
      height={110}
      table={false}
      label="Fan speeds, last 60 seconds"
    />
    <ul class="m-rows">
      {#each fans as f (f.id)}
        <li>
          <div class="m-line">
            <span>{f.label} <span class="m-muted">{f.group}</span></span>
            <b class="tabular">{Math.round(f.rpm)} RPM</b>
          </div>
        </li>
      {/each}
    </ul>
  </MonitorCard>
{/snippet}

<MonitorGrid>
  <MonitorCard title="All temperatures" wide>
    {#snippet value()}{hottest ? m.celsius(hottest.hottest.celsius) : "—"}
      {#if hottest}<small>hottest · {hottest.hottest.label} ({hottest.name})</small>{/if}{/snippet}
    {#snippet aside()}
      <Facts
        items={[
          { label: "Sensors", value: String(all.reduce((n, g) => n + g.readings.length, 0)) },
          { label: "Components", value: String(all.length) },
          { label: "Fans", value: fans.length ? String(fans.length) : null },
        ]}
      />
    {/snippet}
    <LineChart
      series={all.map((g, i) => ({
        name: g.name,
        color: m.colorAt(i),
        values: groupSeries(samples, g.name),
      }))}
      format={m.celsius}
      max={100}
      height={220}
      label="Hottest sensor of each component, last 60 seconds"
    />
  </MonitorCard>

  {#each big as g (g.name)}
    {@render group(g, true)}
  {/each}

  {#each small as item, i (item.group?.name ?? "fans")}
    {#if item.group}
      {@render group(item.group, lastSmallWide(i))}
    {:else}
      {@render fansCard(lastSmallWide(i))}
    {/if}
  {/each}
</MonitorGrid>

<style>
  .cols {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(260px, 1fr));
    column-gap: var(--s-5);
  }
</style>
