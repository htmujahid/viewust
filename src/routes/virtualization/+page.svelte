<script lang="ts">
  import { onMount } from "svelte";

  import { goto } from "$app/navigation";

  import Page from "$lib/components/ui/Page.svelte";
  import RescanButton from "$lib/components/ui/RescanButton.svelte";
  import { osOverview } from "$lib/features/os/overview.svelte";
  import SubsystemCard from "$lib/features/os/SubsystemCard.svelte";
  import { containerFacts, vmFacts } from "$lib/features/os/subsystems";
  import {
    guestCgroupFacts,
    guestNetworkFacts,
    guestStorageFacts,
    hostFacts,
    isolationFacts,
    selfFacts,
  } from "$lib/features/virtualization/facts";

  const ov = osOverview;
  const needed = [ov.virt, ov.containers, ov.vms, ov.namespaces, ov.network];
  const busy = $derived(needed.some((l) => l.loading));

  // The same machine, seen from the guests' side: who can run, who is running,
  // and what they take in storage, cgroups, namespaces and network.
  const cards = $derived([
    {
      href: null,
      icon: "cpu",
      title: "Host capability",
      text: "Whether this processor and kernel can run hardware virtual machines",
      facts: hostFacts(ov.virt.data),
      loading: ov.virt.loading,
      error: ov.virt.error,
    },
    {
      href: "/os/about",
      icon: "overview",
      title: "This machine itself",
      text: "Whether Viewust is looking at bare metal or a guest",
      facts: selfFacts(ov.virt.data),
      loading: ov.virt.loading,
      error: ov.virt.error,
    },
    {
      href: "/virtualization/containers",
      icon: "box",
      title: "Containers",
      text: "Docker or podman: what is running, and the images on disk",
      facts: containerFacts(ov.containers.data),
      loading: ov.containers.loading,
      error: ov.containers.error,
    },
    {
      href: "/virtualization/vms",
      icon: "system",
      title: "Virtual machines",
      text: "Machines defined under libvirt or systemd-machined",
      facts: vmFacts(ov.vms.data),
      loading: ov.vms.loading,
      error: ov.vms.error,
    },
    {
      href: "/os/filesystems",
      icon: "storage",
      title: "Storage, guest side",
      text: "Running containers' overlay roots, and what the runtime holds on disk",
      facts: guestStorageFacts(ov.virt.data),
      loading: ov.virt.loading,
      error: ov.virt.error,
    },
    {
      href: "/os/cgroups",
      icon: "overview",
      title: "Control groups, guest side",
      text: "The slices where guests live, and what they are charged",
      facts: guestCgroupFacts(ov.virt.data),
      loading: ov.virt.loading,
      error: ov.virt.error,
    },
    {
      href: "/processes/namespaces",
      icon: "shield",
      title: "Namespaces, guest side",
      text: "The walls that make a container believe it is alone",
      facts: isolationFacts(ov.namespaces.data),
      loading: ov.namespaces.loading,
      error: ov.namespaces.error,
    },
    {
      href: "/os/network",
      icon: "network",
      title: "Networks, guest side",
      text: "The bridges and virtual interfaces that wire guests to the world",
      facts: guestNetworkFacts(ov.network.data, ov.virt.data),
      loading: ov.network.loading,
      error: ov.network.error,
    },
  ] as const);

  function refresh() {
    for (const l of needed) void l.load();
  }

  function onkeydown(e: KeyboardEvent) {
    if (e.key === "Escape") goto("/");
  }

  onMount(() => needed.forEach((l) => l.ensure()));
</script>

<svelte:window {onkeydown} />

<Page>
  <div class="bar">
    <h1>Virtualization</h1>
    <RescanButton loading={busy} onclick={refresh} label />
  </div>
  <p class="sub">
    Other computers living inside this one — and the storage, control groups, namespaces and
    networks they stand on.
  </p>

  <div class="cards">
    {#each cards as c (c.title)}
      <SubsystemCard
        href={c.href}
        icon={c.icon}
        title={c.title}
        text={c.text}
        facts={c.facts}
        loading={c.loading}
        error={c.error}
      />
    {/each}
  </div>
</Page>

<style>
  .bar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: var(--s-3);
    margin-bottom: var(--s-2);
  }
  h1 {
    font-family: var(--font-mono);
    font-size: 20px;
    font-weight: 700;
    text-shadow: var(--glow-text);
  }
  .sub {
    margin-bottom: var(--s-4);
    color: var(--text-2);
    font-size: var(--fs-small);
  }
  .cards {
    display: grid;
    grid-template-columns: repeat(2, minmax(0, 1fr));
    gap: var(--s-4);
  }
  @media (max-width: 760px) {
    .cards {
      grid-template-columns: minmax(0, 1fr);
    }
  }
</style>
