<script lang="ts">
  import { useSvelteFlow } from "@xyflow/svelte";

  // Re-fit the graph whenever `trigger` changes (e.g. the details panel opens
  // and the canvas gets narrower). Renders nothing.
  let { trigger }: { trigger: unknown } = $props();

  const { fitView } = useSvelteFlow();
  let first = true;

  $effect(() => {
    trigger;
    if (first) {
      first = false;
      return;
    }
    // wait a frame so the canvas has its new size
    const id = requestAnimationFrame(() => fitView({ padding: 0.15, duration: 250 }));
    return () => cancelAnimationFrame(id);
  });
</script>
