<script lang="ts">
  import type { Snippet } from "svelte";

  import SectionShell from "$lib/components/ui/SectionShell.svelte";

  let {
    actions,
    panel,
    scroll = false,
    children,
  }: { actions?: Snippet; panel?: Snippet; scroll?: boolean; children: Snippet } = $props();

  const tabs = [
    { href: "/os", label: "Overview" },
    { href: "/os/modules", label: "Kernel modules" },
    { href: "/os/packages", label: "Packages" },
    { href: "/os/environment", label: "Environment" },
  ];
</script>

<SectionShell {tabs} label="Operating system sections" {actions} {panel} {scroll}>
  {@render children()}
</SectionShell>
