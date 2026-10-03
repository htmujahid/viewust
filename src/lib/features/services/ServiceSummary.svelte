<script lang="ts">
  import type { ServiceOverview } from "$lib/api/types";
  import Meter from "$lib/components/ui/Meter.svelte";

  let { overview: o }: { overview: ServiceOverview } = $props();

  const share = $derived(o.total ? (o.running / o.total) * 100 : 0);
</script>

<section class="tiles">
  <div class="card tile">
    <p class="label">Services</p>
    <p class="value tabular">{o.total} <small>installed</small></p>
    <Meter value={share} tone="accent" />
  </div>
  <div class="card tile">
    <p class="label">Running</p>
    <p class="value tabular ok">{o.running} <small>{o.exited} exited</small></p>
  </div>
  <div class="card tile">
    <p class="label">Failed</p>
    <p class="value tabular" class:bad={o.failed > 0}>
      {o.failed} <small>{o.failed ? "need attention" : "all healthy"}</small>
    </p>
  </div>
  <div class="card tile">
    <p class="label">Start at boot</p>
    <p class="value tabular">{o.enabled} <small>enabled</small></p>
  </div>
</section>

<style>
  .tiles {
    display: grid;
    grid-template-columns: repeat(4, minmax(0, 1fr));
    gap: var(--s-4);
    margin-bottom: var(--s-5);
  }
  @media (max-width: 900px) {
    .tiles {
      grid-template-columns: repeat(2, minmax(0, 1fr));
    }
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
  .value small {
    color: var(--text-2);
    font-size: var(--fs-small);
    font-weight: 400;
  }
  .ok {
    color: var(--ok);
  }
  .bad {
    color: var(--danger);
    text-shadow: 0 0 10px color-mix(in srgb, var(--danger) 55%, transparent);
  }
</style>
