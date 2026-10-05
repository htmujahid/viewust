<script lang="ts">
  import type { Sample } from "$lib/api/types";
  import Meter from "$lib/components/ui/Meter.svelte";
  import * as m from "$lib/features/monitor/series";
  import { formatBytes } from "$lib/utils/format";

  let { sample }: { sample: Sample | null } = $props();

  const gpu = $derived(sample?.gpus.find((g) => g.util !== null) ?? sample?.gpus[0] ?? null);
  const space = $derived.by(() => {
    const volumes = sample?.volumes ?? [];
    const total = volumes.reduce((n, v) => n + v.total, 0);
    const used = volumes.reduce((n, v) => n + v.used, 0);
    return { total, used, pct: total ? (used / total) * 100 : 0 };
  });
</script>

<section class="tiles" aria-label="Live status">
  {#if !sample}
    {#each [0, 1, 2, 3] as i (i)}<div class="skeleton tile-skeleton"></div>{/each}
  {:else}
    <a class="card tile" href="/monitor/cpu">
      <p class="label">Processor</p>
      <p class="value tabular">
        {m.percent(sample.cpu.total)}
        <small
          >{sample.cpu.cores.length} threads{sample.cpu.temperature !== null
            ? ` · ${m.celsius(sample.cpu.temperature)}`
            : ""}</small
        >
      </p>
      <Meter value={sample.cpu.total} />
    </a>
    <a class="card tile" href="/monitor/memory">
      <p class="label">Memory</p>
      <p class="value tabular">
        {formatBytes(sample.memory.used)} <small>of {formatBytes(sample.memory.total)}</small>
      </p>
      <Meter value={(sample.memory.used / sample.memory.total) * 100} />
    </a>
    {#if gpu}
      <a class="card tile" href="/monitor/gpu">
        <p class="label">Graphics</p>
        <p class="value tabular">
          {gpu.util !== null
            ? m.percent(gpu.util)
            : gpu.core_mhz !== null
              ? m.mhz(gpu.core_mhz)
              : "—"}
          <small>{gpu.util !== null ? "load" : gpu.core_mhz !== null ? "clock" : ""}</small>
        </p>
        {#if gpu.util !== null}<Meter value={gpu.util} />{:else}<p class="sub truncate">
            {gpu.name}
          </p>{/if}
      </a>
    {/if}
    <a class="card tile" href="/monitor/storage">
      <p class="label">Disk space</p>
      <p class="value tabular">
        {formatBytes(space.used)} <small>of {formatBytes(space.total)}</small>
      </p>
      <Meter value={space.pct} />
    </a>
  {/if}
</section>

<style>
  .tiles {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(210px, 1fr));
    gap: var(--s-4);
    margin-bottom: var(--s-5);
  }
  .tile {
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: var(--s-2);
    padding: var(--s-4);
    color: inherit;
    text-decoration: none;
  }
  .tile:hover {
    border-color: var(--accent);
    box-shadow: var(--glow);
  }
  .tile-skeleton {
    height: 96px;
  }
  .label {
    color: var(--text-3);
    font-size: var(--fs-label);
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.06em;
  }
  .value {
    font-size: 22px;
    font-weight: 650;
    letter-spacing: -0.01em;
  }
  .value small,
  .sub {
    color: var(--text-2);
    font-size: var(--fs-small);
    font-weight: 400;
  }
</style>
