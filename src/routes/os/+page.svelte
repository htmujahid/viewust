<script lang="ts">
  import { onDestroy, onMount } from "svelte";

  import { goto } from "$app/navigation";

  import Badge from "$lib/components/ui/Badge.svelte";
  import Page from "$lib/components/ui/Page.svelte";
  import { bootedAgo } from "$lib/features/os/logic";
  import { osOverview } from "$lib/features/os/overview.svelte";
  import { os } from "$lib/features/os/store.svelte";
  import SubsystemCard from "$lib/features/os/SubsystemCard.svelte";
  import {
    groupFacts,
    cgroupFacts,
    bootloaderFact,
    envFacts,
    filesystemFacts,
    kernelFacts,
    memoryFacts,
    mountFacts,
    networkFacts,
    namespaceFacts,
    packageFacts,
    processFacts,
    securityFacts,
    serviceFacts,
    summaryFacts,
    userFacts,
  } from "$lib/features/os/subsystems";
  import { formatDuration } from "$lib/utils/format";

  const summary = $derived(os.summary.data);

  // Uptime moves on its own; the rest of the page only changes when asked.
  let now = $state(Date.now() / 1000);
  const tick = setInterval(() => (now = Date.now() / 1000), 30_000);
  onDestroy(() => clearInterval(tick));

  const ov = osOverview;
  // The areas an operating system is made of, each card with its live figures.
  const groups = $derived([
    {
      label: "System",
      text: "What this computer is, how it starts, and who is signed in.",
      cards: [
        {
          href: "/os/about",
          icon: "box",
          title: "Distribution",
          text: "What this system is and where it comes from",
          facts: summaryFacts(summary, [
            ["Operating system", "Distribution"],
            ["Operating system", "Based on"],
            ["Operating system", "Architecture"],
            ["Virtualization", "Runs on"],
          ]),
          loading: os.summary.loading,
          error: os.summary.error,
        },
        {
          href: "/os/session",
          icon: "display",
          title: "Session",
          text: "Who is signed in, their desktop, language and time",
          facts: summaryFacts(summary, [
            ["Session", "Current user"],
            ["Session", "Desktop"],
            ["Session", "Session type"],
            ["Language and time", "Time zone"],
          ]),
          loading: os.summary.loading,
          error: os.summary.error,
        },
        {
          href: "/os/boot",
          icon: "clock",
          title: "Boot",
          text: "Firmware, bootloader, initramfs: how the computer starts",
          facts: [
            ...summaryFacts(summary, [
              ["Firmware", "Interface"],
              ["Firmware", "Version"],
              ["Boot", "Secure Boot"],
              ["Boot", "Started"],
            ]),
            ...bootloaderFact(summary),
          ],
          loading: os.summary.loading,
          error: os.summary.error,
        },
      ],
    },
    {
      label: "Kernel",
      text: "The core of the OS: its drivers, and how it hands memory out.",
      cards: [
        {
          href: "/os/kernel",
          icon: "cpu",
          title: "Kernel & drivers",
          text: "The core of the OS and the driver modules it has loaded",
          facts: kernelFacts(summary, os.modules.data),
          loading: os.summary.loading || os.modules.loading,
          error: os.summary.error,
        },
        {
          href: "/os/memory",
          icon: "memory",
          title: "Memory management",
          text: "RAM, swap, caches and how the kernel hands memory out",
          facts: memoryFacts(os.memory.data),
          loading: os.memory.loading,
          error: os.memory.error,
        },
      ],
    },
    {
      label: "Processes",
      text: "Everything running: how it is supervised, kept apart, and shares the machine.",
      cards: [
        {
          href: "/processes",
          icon: "activity",
          title: "Programs",
          text: "Every running program, scheduled onto the processor",
          facts: processFacts(ov.processes.data),
          loading: ov.processes.loading,
          error: ov.processes.error,
        },
        {
          href: "/services",
          icon: "server",
          title: "Services",
          text: "Programs the OS starts and supervises in the background",
          facts: serviceFacts(ov.services.data),
          loading: ov.services.loading,
          error: ov.services.error,
        },
        {
          href: "/processes/namespaces",
          icon: "system",
          title: "Namespaces",
          text: "The walls that keep programs apart from each other",
          facts: namespaceFacts(ov.namespaces.data),
          loading: ov.namespaces.loading,
          error: ov.namespaces.error,
        },
        {
          href: "/os/cgroups",
          icon: "overview",
          title: "Control groups",
          text: "How processor time, memory and I/O are shared out",
          facts: cgroupFacts(ov.cgroups.data),
          loading: ov.cgroups.loading,
          error: ov.cgroups.error,
        },
      ],
    },
    {
      label: "Storage",
      text: "The drives, the filesystems mounted on them, and what is taking the space.",
      cards: [
        {
          href: "/disk-usage",
          icon: "storage",
          title: "Disk usage",
          text: "Drives, partitions and what is taking the space, folder by folder",
          facts: filesystemFacts(ov.disks.data),
          loading: ov.disks.loading,
          error: ov.disks.error,
        },
        {
          href: "/os/filesystems",
          icon: "storage",
          title: "File systems",
          text: "Everything mounted, from drives to the kernel's own views",
          facts: mountFacts(ov.filesystems.data),
          loading: ov.filesystems.loading,
          error: ov.filesystems.error,
        },
      ],
    },
    {
      label: "Network",
      text: "How the computer talks to the world.",
      cards: [
        {
          href: "/os/network",
          icon: "network",
          title: "Networking",
          text: "Interfaces, addresses, routes, DNS and open ports",
          facts: networkFacts(ov.network.data),
          loading: ov.network.loading,
          error: ov.network.error,
        },
      ],
    },
    {
      label: "Accounts",
      text: "Who may use the computer, and with what rights.",
      cards: [
        {
          href: "/accounts",
          icon: "users",
          title: "Users",
          text: "Who may sign in, and who may administer the computer",
          facts: userFacts(ov.accounts.data),
          loading: ov.accounts.loading,
          error: ov.accounts.error,
        },
        {
          href: "/accounts/groups",
          icon: "users",
          title: "Groups",
          text: "How users are gathered and given shared rights",
          facts: groupFacts(ov.accounts.data),
          loading: ov.accounts.loading,
          error: ov.accounts.error,
        },
      ],
    },
    {
      label: "Security",
      text: "The protections wrapped around all of it.",
      cards: [
        {
          href: "/os/security",
          icon: "shield",
          title: "Security & protection",
          text: "Access control, kernel hardening, Secure Boot and the firewall",
          facts: securityFacts(os.security.data),
          loading: os.security.loading,
          error: os.security.error,
        },
      ],
    },
    {
      label: "Software",
      text: "What is installed, and the variables this session runs with.",
      cards: [
        {
          href: "/os/packages",
          icon: "box",
          title: "Packages",
          text: "Every installed package and the manager behind it",
          facts: packageFacts(os.packages.data),
          loading: os.packages.loading,
          error: os.packages.error,
        },
        {
          href: "/os/environment",
          icon: "terminal",
          title: "Environment",
          text: "The variables this session runs with",
          facts: envFacts(os.environment.data),
          loading: os.environment.loading,
          error: os.environment.error,
        },
      ],
    },
  ] as const);

  function onkeydown(e: KeyboardEvent) {
    if (e.key === "Escape") goto("/");
  }

  onMount(() => {
    os.summary.ensure();
    os.modules.ensure();
    os.memory.ensure();
    os.security.ensure();
    os.packages.ensure();
    os.environment.ensure();
    osOverview.ensure();
  });
</script>

<svelte:window {onkeydown} />

<Page>
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

    {#each groups as g (g.label)}
      <h2 class="heading">{g.label}</h2>
      <p class="layer">{g.text}</p>
      <div class="subsystems">
        {#each g.cards as c (c.title)}
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
    {/each}
  {/if}
</Page>

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
  .heading {
    margin-top: 0;
    margin-bottom: var(--s-1);
    color: var(--text-2);
    font-family: var(--font-mono);
    font-size: var(--fs-label);
    font-weight: 700;
    letter-spacing: 0.06em;
    text-transform: uppercase;
  }
  .layer {
    margin-bottom: var(--s-3);
    color: var(--text-3);
    font-size: var(--fs-small);
  }
  .subsystems {
    display: grid;
    grid-template-columns: repeat(2, minmax(0, 1fr));
    gap: var(--s-4);
    margin-bottom: var(--s-5);
  }
  @media (max-width: 760px) {
    .subsystems {
      grid-template-columns: minmax(0, 1fr);
    }
  }
  .error {
    padding: var(--s-6) 0;
    color: var(--danger);
    text-align: center;
  }
</style>
