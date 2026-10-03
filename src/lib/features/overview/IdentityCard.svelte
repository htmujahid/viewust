<script lang="ts">
  import type { HardwareInfo, Sample, ServiceOverview } from "$lib/api/types";
  import Badge from "$lib/components/ui/Badge.svelte";
  import Illustration from "$lib/illustrations/Illustration.svelte";
  import { readings } from "$lib/features/monitor/thermal";

  import { pick } from "./summary";

  let {
    info,
    sample,
    services,
  }: { info: HardwareInfo | null; sample: Sample | null; services: ServiceOverview | null } =
    $props();

  const rows = $derived(info?.computer_details ?? []);
  const model = $derived(pick(rows, "System", "Model") ?? pick(rows, "System", "Motherboard"));
  const os = $derived(pick(rows, "Operating system", "System"));
  const kernel = $derived(pick(rows, "Operating system", "Kernel"));
  const uptime = $derived(pick(rows, "Operating system", "Uptime"));
  const bios = $derived(pick(rows, "System", "BIOS"));

  const hottest = $derived(
    sample ? Math.max(0, ...readings(sample).map((r) => r.celsius)) || null : null,
  );
  const battery = $derived(sample?.power.batteries[0] ?? null);
</script>

<section class="card hero">
  <div class="art"><Illustration kind="computer" /></div>
  <div class="text">
    <p class="kind">Overview</p>
    <h1>{info?.computer_name ?? "This computer"}</h1>
    <p class="subtitle">{[model, os].filter(Boolean).join(" · ") || "Reading…"}</p>
    <div class="badges">
      {#if services}
        <a href="/services">
          <Badge tone={services.failed ? "danger" : "ok"}>
            {services.failed
              ? `${services.failed} failed ${services.failed === 1 ? "service" : "services"}`
              : "Services healthy"}
          </Badge>
        </a>
      {/if}
      {#if hottest !== null}
        <a href="/monitor/thermal">
          <Badge tone={hottest >= 85 ? "danger" : hottest >= 75 ? "warn" : "neutral"}>
            Hottest {hottest.toFixed(0)} °C
          </Badge>
        </a>
      {/if}
      {#if battery}
        <a href="/monitor/power">
          <Badge tone={battery.percent < 20 ? "danger" : "neutral"}>
            Battery {battery.percent.toFixed(0)}%
          </Badge>
        </a>
      {/if}
      {#if uptime}<Badge>Up {uptime}</Badge>{/if}
      {#if kernel}<Badge>Kernel {kernel}</Badge>{/if}
    </div>
    {#if bios}<p class="bios">BIOS {bios}</p>{/if}
  </div>
</section>

<style>
  .hero {
    display: flex;
    align-items: center;
    gap: var(--s-5);
    padding: var(--s-5);
    margin-bottom: var(--s-4);
  }
  .art {
    position: relative;
    flex: none;
    width: 190px;
    height: 150px;
    border: 1px solid var(--border);
    background:
      linear-gradient(var(--grid-line) 1px, transparent 1px) 0 0 / 16px 16px,
      linear-gradient(90deg, var(--grid-line) 1px, transparent 1px) 0 0 / 16px 16px,
      radial-gradient(circle at 50% 60%, var(--wash-1), transparent 70%),
      var(--surface-2);
  }
  .art :global(svg) {
    position: absolute;
    inset: 12px;
    width: calc(100% - 24px);
    height: calc(100% - 24px);
  }
  .text {
    min-width: 0;
  }
  .kind {
    color: var(--accent);
    font-family: var(--font-mono);
    font-size: var(--fs-label);
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.1em;
  }
  .kind::before {
    content: "> ";
  }
  h1 {
    font-size: 26px;
    font-weight: 650;
    letter-spacing: -0.01em;
    overflow-wrap: anywhere;
    text-shadow: var(--glow-text);
  }
  .subtitle {
    color: var(--text-2);
  }
  .badges {
    display: flex;
    flex-wrap: wrap;
    gap: var(--s-2);
    margin-top: var(--s-3);
  }
  .badges a {
    text-decoration: none;
  }
  .bios {
    margin-top: var(--s-2);
    color: var(--text-3);
    font-family: var(--font-mono);
    font-size: 11px;
  }
  @media (max-width: 640px) {
    .hero {
      flex-direction: column;
      align-items: flex-start;
    }
  }
</style>
