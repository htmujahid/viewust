<script lang="ts">
  import { drawings } from "./drawings";
  import type { Kind } from "./kinds";
  import { artBox, artSize } from "./sizes";

  /**
   * One illustration. The drawing itself lives in `drawings/`; this sizes it,
   * and owns the palette every drawing shares (the `:global` rules below).
   */
  let { kind, cm }: { kind: Kind; cm?: number } = $props();

  const size = $derived(artSize(kind, cm));
  const viewBox = $derived(artBox(kind).join(" "));
  const Drawing = $derived(drawings[kind]);
</script>

<svg {viewBox} width={size.width} height={size.height} role="img" aria-label={kind}>
  <Drawing />
</svg>

<style>
  svg {
    display: block;
    overflow: visible;
  }
  svg :global(*) {
    stroke-linecap: round;
    stroke-linejoin: round;
  }
  svg :global(.body) {
    fill: var(--surface);
    stroke: var(--text-3);
    stroke-width: 1.6;
  }
  svg :global(.paper) {
    fill: var(--surface);
    stroke: var(--text-3);
    stroke-width: 1.6;
  }
  svg :global(.key) {
    fill: var(--surface-2);
    stroke: var(--text-3);
    stroke-width: 1;
  }
  svg :global(.dark) {
    fill: var(--device-dark);
    stroke: var(--text-3);
    stroke-width: 1.4;
  }
  svg :global(.screen) {
    fill: var(--accent-soft);
  }
  svg :global(.accent) {
    fill: var(--accent);
  }
  svg :global(.accent.soft) {
    fill: var(--accent);
    opacity: 0.45;
  }
  svg :global(.soft-fill) {
    fill: #fff;
    opacity: 0.35;
  }
  svg :global(.faint) {
    fill: var(--text-3);
    opacity: 0.45;
  }
  svg :global(.faint-fill) {
    fill: var(--text-3);
    opacity: 0.7;
  }
  svg :global(.metal) {
    fill: var(--surface-2);
    stroke: var(--text-3);
    stroke-width: 1.6;
  }
  svg :global(.pcb) {
    fill: var(--surface-2);
    stroke: var(--text-3);
    stroke-width: 1.6;
  }
  svg :global(.gold) {
    fill: var(--warn);
  }
  svg :global(.ok-fill) {
    fill: var(--ok);
  }
  svg :global(.cloud) {
    fill: var(--surface);
    stroke: var(--text-3);
    stroke-width: 1.8;
  }
  svg :global(.globe) {
    fill: none;
    stroke: var(--accent);
    stroke-width: 2.4;
  }
  svg :global(.glint) {
    fill: #fff;
    opacity: 0.7;
  }
  svg :global(.led) {
    fill: var(--ok);
  }
  svg :global(.line) {
    fill: none;
    stroke: var(--text-3);
    stroke-width: 1.6;
  }
  svg :global(.band) {
    fill: none;
    stroke: var(--text-3);
    stroke-width: 7;
  }
  svg :global(.band.thin) {
    stroke-width: 2.5;
  }
  svg :global(.cable) {
    fill: none;
    stroke: var(--text-3);
    stroke-width: 5;
  }
  svg :global(.glyph) {
    fill: none;
    stroke: #fff;
    stroke-width: 3;
  }
  svg :global(.wave) {
    fill: none;
    stroke: var(--accent);
    stroke-width: 3;
    opacity: 0.6;
  }
</style>
