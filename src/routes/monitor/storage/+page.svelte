<script lang="ts">
  import MonitorCard from "$lib/features/monitor/MonitorCard.svelte";
  import MonitorGrid from "$lib/features/monitor/MonitorGrid.svelte";
  import Facts from "$lib/components/ui/Facts.svelte";
  import LineChart from "$lib/components/charts/LineChart.svelte";
  import Meter from "$lib/components/ui/Meter.svelte";
  import { monitor } from "$lib/features/monitor/store.svelte";
  import * as m from "$lib/features/monitor/series";
  import { formatBytes, formatRate } from "$lib/utils/format";

  const samples = $derived(monitor.samples);
  const now = $derived(monitor.latest)!;
</script>

<MonitorGrid>
  <MonitorCard title="All drives" wide>
    {#snippet value()}{formatRate(m.last(m.diskRead(samples)))} <small>read</small> · {formatRate(
        m.last(m.diskWrite(samples)),
      )} <small>write</small>{/snippet}
    {#snippet aside()}
      <Facts
        items={[
          { label: "Drives", value: String(now.disks.length) },
          { label: "Volumes", value: String(now.volumes.length) },
        ]}
      />
    {/snippet}

    <LineChart
      series={[
        { name: "Read", color: m.COLOR_1, values: m.diskRead(samples) },
        { name: "Write", color: m.COLOR_2, values: m.diskWrite(samples) },
      ]}
      format={formatRate}
      floor={1e6}
      height={200}
      label="Disk read and write speed, last 60 seconds"
    />
  </MonitorCard>

  <MonitorCard title="Space" wide>
    <ul class="m-rows">
      {#each now.volumes as v (v.mount)}
        <li>
          <div class="m-line">
            <span><b>{v.mount}</b> <span class="m-muted">{v.file_system}</span></span>
            <span class="m-muted tabular"
              >{formatBytes(v.used)} used of {formatBytes(v.total)} · {formatBytes(
                v.total - v.used,
              )} free</span
            >
          </div>
          <Meter value={(v.used / v.total) * 100} height={6} />
        </li>
      {/each}
    </ul>
  </MonitorCard>

  {#each now.disks as d, i (d.name)}
    <MonitorCard
      title={`${d.name}${d.model ? ` · ${d.model}` : ""}`}
      wide={now.disks.length % 2 === 1 && i === now.disks.length - 1}
    >
      {#snippet value()}{m.percent(d.busy)} <small>busy</small>{/snippet}
      {#snippet aside()}
        <Facts
          items={[
            { label: "Read", value: formatRate(d.read_bps) },
            { label: "Write", value: formatRate(d.write_bps) },
          ]}
        />
      {/snippet}

      <LineChart
        series={[
          { name: "Read", color: m.COLOR_1, values: m.diskRead(samples, d.name) },
          { name: "Write", color: m.COLOR_2, values: m.diskWrite(samples, d.name) },
        ]}
        format={formatRate}
        floor={1e6}
        height={130}
        table={false}
        label="{d.name} read and write speed, last 60 seconds"
      />
      <div class="busy">
        <span class="m-muted">Time spent busy</span>
        <Meter value={d.busy} tone="accent" height={6} />
      </div>
    </MonitorCard>
  {/each}
</MonitorGrid>

<style>
  .busy {
    margin-top: var(--s-3);
    display: flex;
    flex-direction: column;
    gap: 4px;
  }
</style>
