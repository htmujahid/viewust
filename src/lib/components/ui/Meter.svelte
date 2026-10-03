<script lang="ts">
  let {
    value,
    tone,
    height = 6,
  }: { value: number; tone?: "accent" | "ok" | "warn" | "danger"; height?: number } = $props();

  const auto = $derived(value >= 90 ? "danger" : value >= 75 ? "warn" : "ok");
  const color = $derived(tone ?? auto);
</script>

<div
  class="meter"
  style:height="{height}px"
  role="progressbar"
  aria-valuenow={Math.round(value)}
  aria-valuemin="0"
  aria-valuemax="100"
>
  <div class="fill {color}" style:width="{Math.min(Math.max(value, 0), 100)}%"></div>
</div>

<style>
  .meter {
    border-radius: 0;
    background: var(--surface-2);
    overflow: hidden;
    mask-image: repeating-linear-gradient(90deg, #000 0 5px, transparent 5px 7px);
  }
  .fill {
    height: 100%;
    border-radius: 0;
    transition: width 0.4s ease;
  }
  .accent {
    background: var(--accent);
  }
  .ok {
    background: var(--ok);
  }
  .warn {
    background: var(--warn);
  }
  .danger {
    background: var(--danger);
  }
</style>
