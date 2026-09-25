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
    padding: 3px 10px;
    border-radius: 999px;
    font-size: var(--fs-small);
    font-weight: 600;
    white-space: nowrap;
  }
  .neutral {
    background: var(--surface-2);
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
