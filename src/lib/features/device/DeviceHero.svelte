<script lang="ts">
  import Badge from "$lib/components/ui/Badge.svelte";
  import Illustration from "$lib/illustrations/Illustration.svelte";
  import type { NodeInfo } from "$lib/map/model";

  let { info }: { info: NodeInfo } = $props();
</script>

<section class="hero card">
  <div class="art"><Illustration kind={info.kind} /></div>
  <div class="text">
    <p class="kind">{info.label}</p>
    <h1>{info.title}</h1>
    {#if info.subtitle}<p class="subtitle">{info.subtitle}</p>{/if}
    <div class="badges">
      {#if info.connection}
        <Badge tone={info.connection === "Wireless" ? "ok" : "neutral"}>{info.connection}</Badge>
      {/if}
      {#if info.via}<Badge>via {info.via}</Badge>{/if}
    </div>
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
    height: 130px;
    border-radius: 0;
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
    font-size: var(--fs-label);
    font-weight: 600;
    font-family: var(--font-mono);
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
    text-shadow: var(--glow-text);
    overflow-wrap: anywhere;
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

  @media (max-width: 640px) {
    .hero {
      flex-direction: column;
      align-items: flex-start;
    }
  }
</style>
