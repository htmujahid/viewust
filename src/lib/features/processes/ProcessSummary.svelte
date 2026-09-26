<script lang="ts">
  import type { Overview } from "$lib/api/types";
  import Meter from "$lib/components/ui/Meter.svelte";
  import { formatBytes, formatDuration } from "$lib/utils/format";

  let { overview: o }: { overview: Overview } = $props();

  const memPct = $derived((o.memory_used / o.memory_total) * 100);
  const swapPct = $derived(o.swap_total ? (o.swap_used / o.swap_total) * 100 : 0);
</script>

<section class="tiles">
  <div class="card tile">
    <p class="label">Memory</p>
    <p class="value tabular">
      {formatBytes(o.memory_used)} <small>of {formatBytes(o.memory_total)}</small>
    </p>
    <Meter value={memPct} />
  </div>
  <div class="card tile">
    <p class="label">CPU</p>
    <!-- machine-wide load is already 0–100 across every thread -->
    <p class="value tabular">{o.cpu.toFixed(0)}% <small>across {o.cpu_count} threads</small></p>
    <Meter value={o.cpu} />
  </div>
  <div class="card tile">
    <p class="label">Swap</p>
    <p class="value tabular">
      {o.swap_total ? formatBytes(o.swap_used) : "None"}
      {#if o.swap_total}<small>of {formatBytes(o.swap_total)}</small>{/if}
    </p>
    <Meter value={swapPct} />
  </div>
  <div class="card tile">
    <p class="label">Load average</p>
    <p class="value tabular">
      {o.load[0].toFixed(2)} <small>{o.load[1].toFixed(2)} · {o.load[2].toFixed(2)}</small>
    </p>
    <p class="sub">1, 5 and 15 minutes · up {formatDuration(o.uptime)}</p>
  </div>
</section>

<style>
  .tiles {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(210px, 1fr));
    gap: var(--s-4);
    margin-bottom: var(--s-5);
  }
  .tile {
    display: flex;
    flex-direction: column;
    gap: var(--s-2);
    padding: var(--s-4);
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
