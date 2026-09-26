<script lang="ts">
  import { onMount } from "svelte";

  import { goto } from "$app/navigation";
  import { page } from "$app/state";

  import CopyButton from "$lib/components/ui/CopyButton.svelte";
  import Page from "$lib/components/ui/Page.svelte";
  import PageHeader from "$lib/components/ui/PageHeader.svelte";
  import DeviceHero from "$lib/features/device/DeviceHero.svelte";
  import { DeviceReport } from "$lib/features/device/report.svelte";
  import SectionCards from "$lib/features/device/SectionCards.svelte";
  import SectionNav from "$lib/features/device/SectionNav.svelte";
  import { hardware } from "$lib/features/devices/store.svelte";
  import { internals } from "$lib/features/internals/store.svelte";
  import { infoRows, type NodeInfo } from "$lib/map/model";
  import { groupBySection, sectionsToText, sortSections } from "$lib/utils/details";

  /** Identity first, then wiring and power, then the deep technical sections. */
  const SECTION_ORDER = [
    "Device",
    "Monitor",
    "System",
    "Panel",
    "Identification",
    "Connection",
    "Power",
    "Registry",
  ];

  const id = $derived(decodeURIComponent(page.params.id ?? ""));

  // Internal parts (ids start with "sys:") live in their own store.
  const internal = $derived(id.startsWith("sys:"));
  const store = $derived(internal ? internals : hardware);
  const backUrl = $derived(internal ? "/system" : "/");

  const info = $derived<NodeInfo | null>(
    (store.nodes.find((n) => n.id === id)?.data?.info as NodeInfo | undefined) ?? null,
  );

  const report = new DeviceReport();

  // What the map already knew comes first, then the deep report; same-named sections merge.
  const sections = $derived(
    sortSections(
      groupBySection([...(info ? infoRows(info) : []), ...(report.rows ?? [])]),
      SECTION_ORDER,
    ),
  );

  onMount(() => {
    if (id === "computer") goto("/system", { replaceState: true });
    else store.ensure();
  });

  // Read the deep report whenever the device in the URL changes.
  $effect(() => {
    report.load(id);
  });

  function onkeydown(e: KeyboardEvent) {
    if (e.key === "Escape") goto(backUrl);
  }
</script>

<svelte:window {onkeydown} />

<Page>
  <PageHeader back={backUrl} backLabel={internal ? "Internal components" : "All devices"}>
    {#snippet actions()}
      <CopyButton
        label="Copy report"
        disabled={!sections.length}
        text={() => sectionsToText(`${info?.label ?? ""}: ${info?.title ?? id}`, sections)}
      />
    {/snippet}
  </PageHeader>

  {#if !info && store.loading}
    <p class="note">Scanning…</p>
  {:else if !info}
    <div class="card note">
      <p>This device isn't connected any more.</p>
      <a class="btn" href={backUrl}>Go back</a>
    </div>
  {:else}
    <DeviceHero {info} />
    {#if sections.length > 1}<SectionNav {sections} />{/if}
    <SectionCards {sections} pending={report.pending} error={report.error} />
  {/if}
</Page>

<style>
  .note {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: var(--s-3);
    padding: var(--s-6);
    color: var(--text-2);
    text-align: center;
  }
</style>
