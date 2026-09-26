<script lang="ts">
  import { Handle, Position } from "@xyflow/svelte";

  let { inputs = false, outputs = false }: { inputs?: boolean; outputs?: boolean } = $props();

  const sides = [
    ["t", Position.Top],
    ["b", Position.Bottom],
    ["l", Position.Left],
    ["r", Position.Right],
  ] as const;
</script>

{#each sides as [id, position]}
  {#if inputs}<Handle id="in-{id}" type="target" {position} />{/if}
  {#if outputs}<Handle id="out-{id}" type="source" {position} />{/if}
{/each}
