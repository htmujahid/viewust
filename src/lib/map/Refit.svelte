<script lang="ts">
  import { useSvelteFlow, type FitViewOptions } from "@xyflow/svelte";

  let { trigger, padding = 0.15 }: { trigger: unknown; padding?: FitViewOptions["padding"] } =
    $props();

  const { fitView } = useSvelteFlow();
  let first = true;

  $effect(() => {
    trigger;
    if (first) {
      first = false;
      return;
    }
    const id = requestAnimationFrame(() => fitView({ padding, duration: 250 }));
    return () => cancelAnimationFrame(id);
  });
</script>
