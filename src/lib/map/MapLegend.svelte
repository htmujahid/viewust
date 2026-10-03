<script lang="ts" module>
  export type LineKind = "wired" | "wireless" | "memory" | "power";
</script>

<script lang="ts">
  let { items }: { items: { label: string; kind: LineKind }[] } = $props();
</script>

<div class="card legend">
  {#each items as item}
    <span><i class="line {item.kind}"></i> {item.label}</span>
  {/each}
</div>

<style>
  .legend {
    display: flex;
    gap: var(--s-4);
    padding: var(--s-2) var(--s-3);
    color: var(--text-2);
    font-size: var(--fs-small);
  }
  .legend span {
    display: inline-flex;
    align-items: center;
    gap: var(--s-2);
  }
  .line {
    display: inline-block;
    width: 28px;
    height: 0;
    border-top: 3px solid var(--text-3);
  }
  .line.wireless {
    border-top: 3px dotted var(--accent);
  }
  .line.memory {
    border-top-color: var(--accent);
  }
  .line.power {
    border-top: 4px solid var(--warn);
  }
</style>
