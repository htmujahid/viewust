<script lang="ts">
  import { onDestroy, onMount } from "svelte";

  import { goto } from "$app/navigation";

  import Badge from "$lib/components/ui/Badge.svelte";
  import DetailList from "$lib/components/ui/DetailList.svelte";
  import Icon, { type IconName } from "$lib/components/ui/Icon.svelte";
  import Masonry from "$lib/components/ui/Masonry.svelte";
  import RescanButton from "$lib/components/ui/RescanButton.svelte";
  import { bootedAgo } from "$lib/features/os/logic";
  import OsShell from "$lib/features/os/OsShell.svelte";
  import { os } from "$lib/features/os/store.svelte";
  import { groupBySection } from "$lib/utils/details";
  import { formatDuration } from "$lib/utils/format";
  import { sectionCost } from "$lib/utils/masonry";

  const summary = $derived(os.summary.data);
  const items = $derived(
    groupBySection(summary?.details ?? []).map((section) => ({
      key: section.title,
      cost: sectionCost(section.rows),
      section,
    })),
  );

  // Uptime moves on its own; the rest of the page only changes when asked.
  let now = $state(Date.now() / 1000);
  const tick = setInterval(() => (now = Date.now() / 1000), 30_000);
  onDestroy(() => clearInterval(tick));

  const related: { href: string; icon: IconName; title: string; text: string }[] = [
    {
      href: "/processes",
      icon: "activity",
      title: "Processes",
      text: "Programs running now and the memory they use",
    },
    {
      href: "/processes/namespaces",
      icon: "system",
      title: "Namespaces",
      text: "How programs are kept apart from each other",
    },
    {
      href: "/services",
      icon: "server",
      title: "Services",
      text: "Background services, their state and logs",
    },
    { href: "/accounts", icon: "users", title: "Users", text: "Accounts on this computer" },
    {
      href: "/accounts/groups",
      icon: "users",
      title: "Groups",
      text: "Groups and who belongs to them",
    },
    {
      href: "/disk-usage",
      icon: "storage",
      title: "Disk usage",
      text: "What is taking the space on every drive",
    },
    {
      href: "/monitor",
      icon: "chart",
      title: "Live monitor",
      text: "Processor, memory, disk and network in real time",
    },
    {
      href: "/system",
      icon: "cpu",
      title: "Hardware inside",
      text: "The parts this system runs on",
    },
  ];

  function onkeydown(e: KeyboardEvent) {
    if (e.key === "Escape") goto("/");
  }

  onMount(os.summary.ensure);
</script>

<svelte:window {onkeydown} />

<OsShell scroll>
  {#snippet actions()}
    <RescanButton loading={os.summary.loading} onclick={os.summary.load} label />
  {/snippet}

  {#if os.summary.error && !summary}
    <p class="error">Couldn't read the operating system: {os.summary.error}</p>
  {:else if !summary}
    <div class="skeleton" style="height: 140px; margin-bottom: 16px"></div>
    <div class="skeleton" style="height: 420px"></div>
  {:else}
    <section class="card hero">
      <p class="eyebrow">&gt; Operating system</p>
      <h1>{summary.name}</h1>
      {#if summary.version}<p class="sub">{summary.version}</p>{/if}
      <div class="chips">
        <Badge>Kernel {summary.kernel}</Badge>
        <Badge>{summary.architecture}</Badge>
        <Badge>{summary.hostname}</Badge>
        <Badge tone="ok">Up {formatDuration(bootedAgo(summary.boot_time, now))}</Badge>
      </div>
    </section>

    <Masonry {items}>
      {#snippet children(item)}
        <section class="card block">
          <h2>{item.section.title}</h2>
          <DetailList rows={item.section.rows} stackAt={64} />
        </section>
      {/snippet}
    </Masonry>

    <h2 class="heading">Related</h2>
    <div class="related">
      {#each related as r (r.href)}
        <a class="card link" href={r.href}>
          <Icon name={r.icon} size={20} />
          <span>
            <b>{r.title}</b>
            <small>{r.text}</small>
          </span>
        </a>
      {/each}
    </div>
  {/if}
</OsShell>

<style>
  .hero {
    margin-bottom: var(--s-5);
    padding: var(--s-5);
  }
  .eyebrow {
    color: var(--accent);
    font-family: var(--font-mono);
    font-size: var(--fs-label);
    font-weight: 600;
    letter-spacing: 0.06em;
    text-transform: uppercase;
  }
  h1 {
    margin: var(--s-2) 0 var(--s-1);
    font-family: var(--font-mono);
    font-size: var(--fs-title);
    font-weight: 700;
    text-shadow: var(--glow-text);
  }
  .sub {
    color: var(--text-2);
  }
  .chips {
    display: flex;
    flex-wrap: wrap;
    gap: var(--s-2);
    margin-top: var(--s-3);
  }
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
  .heading {
    margin-top: var(--s-4);
    margin-bottom: var(--s-3);
    color: var(--text-2);
    font-family: var(--font-mono);
    font-size: var(--fs-label);
    font-weight: 700;
    letter-spacing: 0.06em;
    text-transform: uppercase;
  }
  .related {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(260px, 1fr));
    gap: var(--s-3);
    margin-bottom: var(--s-5);
  }
  .link {
    display: flex;
    align-items: flex-start;
    gap: var(--s-3);
    margin: 0;
    padding: var(--s-3) var(--s-4);
    color: var(--text);
    text-decoration: none;
  }
  .link:hover {
    border-color: var(--accent);
    box-shadow: var(--glow);
  }
  .link :global(svg) {
    flex: none;
    margin-top: 2px;
    color: var(--accent);
  }
  .link span {
    display: flex;
    flex-direction: column;
    gap: 2px;
    min-width: 0;
  }
  .link small {
    color: var(--text-2);
    font-size: var(--fs-small);
  }
  .error {
    padding: var(--s-6) 0;
    color: var(--danger);
    text-align: center;
  }
</style>
