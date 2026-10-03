<script lang="ts">
  import type { Snippet } from "svelte";

  import SectionShell from "$lib/components/ui/SectionShell.svelte";

  let { actions, panel, children }: { actions?: Snippet; panel?: Snippet; children: Snippet } =
    $props();

  const tabs = [
    { href: "/processes", label: "Programs" },
    { href: "/processes/namespaces", label: "Namespaces" },
  ];
</script>

<SectionShell {tabs} label="Process sections" {actions} {panel}>{@render children()}</SectionShell>
