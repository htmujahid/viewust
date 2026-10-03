<script lang="ts">
  import type { Snippet } from "svelte";

  let {
    tone = "neutral",
    children,
  }: { tone?: "neutral" | "ok" | "warn" | "danger"; children: Snippet } = $props();
</script>

<span class="badge {tone} tabular">{@render children()}</span>

<style>
  .badge {
    display: inline-block;
    padding: 2px 8px;
    border: 1px solid currentColor;
    border-radius: 0;
    font-family: var(--font-mono);
    font-size: 11px;
    font-weight: 600;
    letter-spacing: 0.05em;
    text-transform: uppercase;
    white-space: nowrap;
  }
  .neutral {
    background: var(--surface-2);
    border-color: var(--border-strong);
    color: var(--text-2);
  }
  .ok {
    background: var(--ok-soft);
    color: var(--ok);
  }
  .warn {
    background: var(--warn-soft);
    color: var(--warn);
  }
  .danger {
    background: var(--danger-soft);
    color: var(--danger);
  }
</style>
