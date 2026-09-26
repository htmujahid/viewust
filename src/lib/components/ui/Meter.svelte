<script lang="ts">
  /** A thin horizontal bar. `value` is 0–100; `tone` overrides the automatic colour. */
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
    border-radius: 999px;
    background: var(--surface-2);
    overflow: hidden;
  }
  .fill {
    height: 100%;
    border-radius: inherit;
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
