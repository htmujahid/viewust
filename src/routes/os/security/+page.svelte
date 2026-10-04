<script lang="ts">
  import { onMount } from "svelte";

  import { goto } from "$app/navigation";

  import DetailList from "$lib/components/ui/DetailList.svelte";
  import Masonry from "$lib/components/ui/Masonry.svelte";
  import RescanButton from "$lib/components/ui/RescanButton.svelte";
  import StatTiles from "$lib/components/ui/StatTiles.svelte";
  import SectionShell from "$lib/components/ui/SectionShell.svelte";
  import { os } from "$lib/features/os/store.svelte";
  import { groupBySection } from "$lib/utils/details";
  import { sectionCost } from "$lib/utils/masonry";

  const data = $derived(os.security.data);
  const items = $derived(
    groupBySection(data?.details ?? []).map((section) => ({
      key: section.title,
      cost: sectionCost(section.rows),
      section,
    })),
  );

  const good = (value: string | null, want: string) =>
    value === want ? ("ok" as const) : value === null ? undefined : ("warn" as const);

  function onkeydown(e: KeyboardEvent) {
    if (e.key === "Escape") goto("/os");
  }

  onMount(os.security.ensure);
</script>

<svelte:window {onkeydown} />

<SectionShell scroll title="Security & protection">
  {#snippet actions()}
    <RescanButton loading={os.security.loading} onclick={os.security.load} label />
  {/snippet}

  {#if os.security.error && !data}
    <p class="error">Couldn't read the protections: {os.security.error}</p>
  {:else if !data}
    <div class="skeleton" style="height: 96px; margin-bottom: 16px"></div>
    <div class="skeleton" style="height: 320px"></div>
  {:else}
    <StatTiles
      items={[
        {
          label: "Access control",
          value: data.apparmor ? "AppArmor" : data.selinux ? "SELinux" : "None",
          sub: data.apparmor ?? data.selinux ?? "not detected",
          tone: data.apparmor === "Enabled" || data.selinux === "Enforcing" ? "ok" : "warn",
        },
        {
          label: "Secure Boot",
          value: data.secure_boot ?? "—",
          sub: data.secure_boot ? undefined : "legacy BIOS",
          tone: good(data.secure_boot, "Enabled"),
        },
        {
          label: "Kernel lockdown",
          value: data.lockdown ?? "off",
          tone: data.lockdown && data.lockdown !== "none" ? "ok" : undefined,
        },
      ]}
    />

    <Masonry {items}>
      {#snippet children(item)}
        <section class="card block">
          <h2>{item.section.title}</h2>
          <DetailList rows={item.section.rows} stackAt={64} />
        </section>
      {/snippet}
    </Masonry>
  {/if}
</SectionShell>

<style>
  .block {
    margin: 0;
    padding: var(--s-4) var(--s-5);
  }
  .block h2 {
    margin-bottom: var(--s-3);
    color: var(--accent);
    font-family: var(--font-mono);
    font-size: var(--fs-label);
    font-weight: 700;
    letter-spacing: 0.06em;
    text-transform: uppercase;
  }
  .error {
    padding: var(--s-6) 0;
    color: var(--danger);
    text-align: center;
  }
</style>
