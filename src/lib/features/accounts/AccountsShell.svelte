<script lang="ts">
  import type { Snippet } from "svelte";

  import SectionShell from "$lib/components/ui/SectionShell.svelte";

  let { actions, panel, children }: { actions?: Snippet; panel?: Snippet; children: Snippet } =
    $props();

  const tabs = [
    { href: "/accounts", label: "Users" },
    { href: "/accounts/groups", label: "Groups" },
  ];
</script>

<SectionShell {tabs} label="Account sections" {actions} {panel}>{@render children()}</SectionShell>
