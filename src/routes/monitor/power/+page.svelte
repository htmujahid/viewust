<script lang="ts">
  import type { Battery } from "$lib/api/types";
  import LineChart from "$lib/components/charts/LineChart.svelte";
  import Badge from "$lib/components/ui/Badge.svelte";
  import Facts from "$lib/components/ui/Facts.svelte";
  import MonitorCard from "$lib/features/monitor/MonitorCard.svelte";
  import MonitorGrid from "$lib/features/monitor/MonitorGrid.svelte";
  import * as m from "$lib/features/monitor/series";
  import { monitor } from "$lib/features/monitor/store.svelte";
  import { formatDuration } from "$lib/utils/format";

  const samples = $derived(monitor.samples);
  const now = $derived(monitor.latest)!;
  const power = $derived(now.power);
  const gpus = $derived(now.gpus.filter((g) => g.power !== null));

  const sources = $derived([
    ...(power.cpu_readable
      ? [{ name: "Processor", color: m.colorAt(0), values: m.cpuWatts(samples) }]
      : []),
    ...gpus.map((g, i) => ({
      name: g.name,
      color: m.colorAt(i + (power.cpu_readable ? 1 : 0)),
      values: m.gpuPower(samples, g.id),
    })),
  ]);
  const measured = $derived((power.cpu_watts ?? 0) + gpus.reduce((n, g) => n + (g.power ?? 0), 0));

  const keys = $derived([
    "cpu",
    ...gpus.map((g) => `gpu:${g.id}`),
    ...power.batteries.map((b) => `bat:${b.name}`),
    "energy",
  ]);
  const wide = (key: string) => keys.length % 2 === 1 && keys.at(-1) === key;

  const energy = $derived(monitor.energy);
  const totalWh = $derived(energy.cpu + gpus.reduce((n, g) => n + (energy.gpus[g.id] ?? 0), 0));
  const wh = (v: number) => (v < 10 ? `${v.toFixed(2)} Wh` : `${v.toFixed(1)} Wh`);

  const ac = $derived(
    power.ac_online === null ? null : power.ac_online ? "Plugged in" : "On battery",
  );
  const statusTone = (b: Battery) =>
    b.status === "Charging" || b.status === "Full" ? "ok" : b.percent < 20 ? "danger" : "neutral";
</script>

<MonitorGrid>
  {#if sources.length}
    <MonitorCard title="Power draw" wide>
      {#snippet value()}{m.watts(measured)}
        <small>measured{power.cpu_readable ? " · processor + graphics" : " · graphics only"}</small
        >{/snippet}
      {#snippet aside()}
        <Facts
          items={[
            { label: "Mains", value: ac },
            {
              label: "Processor limit",
              value:
                power.cpu_limit_sustained !== null
                  ? `${m.watts(power.cpu_limit_sustained)}${power.cpu_limit_boost ? ` · boost ${m.watts(power.cpu_limit_boost)}` : ""}`
                  : null,
            },
          ]}
        />
      {/snippet}
      <LineChart
        series={sources}
        format={m.watts}
        floor={20}
        height={200}
        label="Measured power draw, last 60 seconds"
      />
    </MonitorCard>
  {/if}

  <MonitorCard title="Processor" wide={wide("cpu")}>
    {#snippet value()}{power.cpu_readable
        ? power.cpu_watts !== null
          ? m.watts(power.cpu_watts)
          : "—"
        : power.cpu_limit_sustained !== null
          ? m.watts(power.cpu_limit_sustained)
          : "—"}
      <small>{power.cpu_readable ? "package" : "sustained limit"}</small>{/snippet}
    {#snippet aside()}
      <Facts
        items={[
          {
            label: power.cpu_readable ? "Sustained limit" : "Boost limit",
            value: power.cpu_readable
              ? power.cpu_limit_sustained !== null
                ? m.watts(power.cpu_limit_sustained)
                : null
              : power.cpu_limit_boost !== null
                ? m.watts(power.cpu_limit_boost)
                : null,
          },
          {
            label: "Boost limit",
            value:
              power.cpu_readable && power.cpu_limit_boost !== null
                ? m.watts(power.cpu_limit_boost)
                : null,
          },
        ]}
      />
    {/snippet}
    {#if power.cpu_readable}
      <LineChart
        series={[{ name: "Processor", color: m.COLOR_1, values: m.cpuWatts(samples) }]}
        format={m.watts}
        max={power.cpu_limit_boost ?? undefined}
        floor={20}
        height={150}
        label="Processor power draw, last 60 seconds"
      />
    {:else}
      <p class="m-muted note">
        How much the processor is drawing right now can't be read here: Linux only lets an
        administrator read the processor's energy counter. The limits shown are what the firmware
        allows it to draw.
      </p>
    {/if}
  </MonitorCard>

  {#each gpus as g (g.id)}
    {@const draw = g.power as number}
    <MonitorCard title={g.name} wide={wide(`gpu:${g.id}`)}>
      {#snippet value()}{m.watts(draw)}{#if g.power_limit}
          <small>of {m.watts(g.power_limit)} limit</small>{/if}{/snippet}
      <LineChart
        series={[{ name: "Power", color: m.COLOR_1, values: m.gpuPower(samples, g.id) }]}
        format={m.watts}
        max={g.power_limit ?? undefined}
        height={150}
        label="{g.name} power draw, last 60 seconds"
      />
    </MonitorCard>
  {/each}

  {#each power.batteries as b (b.name)}
    <MonitorCard title={`Battery · ${b.name}`} wide={wide(`bat:${b.name}`)}>
      {#snippet value()}{b.percent.toFixed(0)}%
        <small><Badge tone={statusTone(b)}>{b.status}</Badge></small>{/snippet}
      {#snippet aside()}
        <Facts
          items={[
            { label: "Rate", value: b.watts !== null ? m.watts(b.watts) : null },
            {
              label: b.status === "Charging" ? "Full in" : "Time left",
              value: b.seconds_left !== null ? formatDuration(b.seconds_left) : null,
            },
            { label: "Health", value: b.health !== null ? `${b.health.toFixed(0)}%` : null },
            { label: "Cycles", value: b.cycles !== null ? String(b.cycles) : null },
          ]}
        />
      {/snippet}
      <LineChart
        series={[{ name: "Charge", color: m.COLOR_1, values: m.batteryPercent(samples, b.name) }]}
        format={m.percent}
        max={100}
        height={150}
        label="{b.name} charge, last 60 seconds"
      />
    </MonitorCard>
  {/each}

  <MonitorCard title="Energy this session" wide={wide("energy")}>
    {#snippet value()}{wh(totalWh)} <small>in {formatDuration(energy.seconds)}</small>{/snippet}
    <ul class="m-rows">
      {#if power.cpu_readable}
        <li>
          <div class="m-line"><span>Processor</span><b class="tabular">{wh(energy.cpu)}</b></div>
        </li>
      {/if}
      {#each gpus as g (g.id)}
        <li>
          <div class="m-line">
            <span>{g.name}</span><b class="tabular">{wh(energy.gpus[g.id] ?? 0)}</b>
          </div>
        </li>
      {/each}
    </ul>
    <p class="m-muted note">
      Counted from the readings above since this window opened. Drives, fans, memory and the board
      draw more than this.
    </p>
  </MonitorCard>
</MonitorGrid>

<style>
  .note {
    margin-top: var(--s-3);
  }
</style>
