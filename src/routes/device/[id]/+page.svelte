<script lang="ts">
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { goto } from "$app/navigation";
  import { page } from "$app/state";
  import Badge from "$lib/components/Badge.svelte";
  import ThemeToggle from "$lib/components/ThemeToggle.svelte";
  import Illustration from "$lib/flow/Illustration.svelte";
  import { hardware } from "$lib/hardware.svelte";
  import { internals } from "$lib/internals.svelte";
  import type { NodeInfo } from "$lib/flow/graph";
  import type { Detail } from "$lib/system";

  const id = $derived(decodeURIComponent(page.params.id ?? ""));
  // Internal parts (ids start with "sys:") live in their own store.
  const internal = $derived(id.startsWith("sys:"));
  const store = $derived(internal ? internals : hardware);
  const info = $derived<NodeInfo | null>(
    (store.nodes.find((n) => n.id === id)?.data?.info as NodeInfo | undefined) ?? null,
  );
  const backUrl = $derived(internal ? "/system" : "/");

  let report = $state<Detail[] | null>(null);
  let reportError = $state<string | null>(null);
  let copied = $state(false);

  onMount(() => {
    if (id === "computer") goto("/system", { replaceState: true });
    else store.ensure();
  });

  // Fetch the deep report whenever the device in the URL changes.
  $effect(() => {
    const current = id;
    report = null;
    reportError = null;
    invoke<Detail[]>("device_report", { id: current })
      .then((r) => current === id && (report = r))
      .catch((e) => (reportError = String(e)));
  });

  // Facts from the graph first (summary), then the deep report; same-named
  // sections merge. Order of first appearance is kept.
  const sections = $derived.by(() => {
    const rows: Detail[] = [
      ...(info?.via
        ? [{ section: "Connection", label: "Connected through", value: info.via }]
        : []),
      ...(info?.details ?? []),
      ...(report ?? []),
    ];
    const groups: { title: string; rows: Detail[] }[] = [];
    for (const r of rows) {
      let g = groups.find((x) => x.title === r.section);
      if (!g) groups.push((g = { title: r.section, rows: [] }));
      g.rows.push(r);
    }
    // identity first, then wiring and power, then the deep technical sections
    const order = ["Device", "Monitor", "System", "Panel", "Identification", "Connection", "Power", "Registry"];
    const rank = (t: string) => (order.includes(t) ? order.indexOf(t) : order.length);
    return groups
      .map((g, i) => ({ g, i }))
      .sort((a, b) => rank(a.g.title) - rank(b.g.title) || a.i - b.i)
      .map((x) => x.g);
  });

  const slug = (t: string) => "s-" + t.toLowerCase().replace(/[^a-z0-9]+/g, "-");
  const long = (v: string) => v.length > 64 || v.includes("\n");

  async function copyAll() {
    const text = [
      `${info?.label ?? ""}: ${info?.title ?? id}`,
      ...sections.flatMap((s) => [`\n[${s.title}]`, ...s.rows.map((r) => `${r.label}: ${r.value}`)]),
    ].join("\n");
    try {
      await navigator.clipboard.writeText(text);
      copied = true;
      setTimeout(() => (copied = false), 1500);
    } catch {
      // clipboard unavailable
    }
  }

  function onkeydown(e: KeyboardEvent) {
    if (e.key === "Escape") goto(backUrl);
  }
</script>

<svelte:window {onkeydown} />

<div class="page">
  <header class="top">
    <button class="btn" onclick={() => goto(backUrl)}>
      <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M19 12H5M11 6l-6 6 6 6" /></svg>
      {internal ? "Internal components" : "All devices"}
    </button>
    <div class="spacer"></div>
    <button class="btn" onclick={copyAll} disabled={!sections.length}>
      {copied ? "Copied" : "Copy report"}
    </button>
    <ThemeToggle />
  </header>

  {#if !info && store.loading}
    <p class="note">Scanning…</p>
  {:else if !info}
    <div class="card note">
      <p>This device isn't connected any more.</p>
      <button class="btn" onclick={() => goto(backUrl)}>Go back</button>
    </div>
  {:else}
    <section class="hero card">
      <div class="art"><Illustration kind={info.kind} /></div>
      <div class="hero-text">
        <p class="kind">{info.label}</p>
        <h1>{info.title}</h1>
        {#if info.subtitle}<p class="subtitle">{info.subtitle}</p>{/if}
        <div class="badges">
          {#if info.connection}
            <Badge tone={info.connection === "Wireless" ? "ok" : "neutral"}>{info.connection}</Badge>
          {/if}
          {#if info.via}<Badge>via {info.via}</Badge>{/if}
        </div>
      </div>
    </section>

    {#if sections.length > 1}
      <nav class="toc" aria-label="Sections">
        {#each sections as s}
          <a href="#{slug(s.title)}">{s.title}</a>
        {/each}
      </nav>
    {/if}

    <div class="columns">
      {#each sections as s (s.title)}
        <section class="card block" id={slug(s.title)}>
          <h2>{s.title}</h2>
          <dl>
            {#each s.rows as r}
              <div class="row" class:stacked={long(r.value)}>
                <dt>{r.label}</dt>
                {#if r.value.includes("\n")}
                  <dd><pre>{r.value}</pre></dd>
                {:else}
                  <dd>{r.value}</dd>
                {/if}
              </div>
            {/each}
          </dl>
        </section>
      {/each}

      {#if report === null && !reportError}
        <section class="card block">
          <h2>Technical details</h2>
          <p class="muted">Reading the device…</p>
        </section>
      {/if}
      {#if reportError}
        <section class="card block">
          <h2>Technical details</h2>
          <p class="error">Couldn't read them: {reportError}</p>
        </section>
      {/if}
    </div>
  {/if}
</div>

<style>
  .page {
    height: 100vh;
    overflow-y: auto;
    padding: var(--s-4) var(--s-5) var(--s-6);
    scroll-behavior: smooth;
  }
  .page > :global(*) {
    max-width: 1100px;
    margin-left: auto;
    margin-right: auto;
  }

  .top {
    display: flex;
    align-items: center;
    gap: var(--s-3);
    margin-bottom: var(--s-4);
  }
  .spacer {
    flex: 1;
  }
  .btn svg {
    flex: none;
  }

  .hero {
    display: flex;
    align-items: center;
    gap: var(--s-5);
    padding: var(--s-5);
    margin-bottom: var(--s-4);
  }
  .art {
    position: relative;
    flex: none;
    width: 190px;
    height: 130px;
    border-radius: var(--radius);
    background: radial-gradient(circle at 50% 45%, var(--accent-soft), var(--surface-2) 90%);
  }
  .art :global(svg) {
    position: absolute;
    inset: 12px;
    width: calc(100% - 24px);
    height: calc(100% - 24px);
  }
  .hero-text {
    min-width: 0;
  }
  .kind {
    color: var(--accent);
    font-size: var(--fs-label);
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.06em;
  }
  h1 {
    font-size: 26px;
    font-weight: 650;
    letter-spacing: -0.01em;
    overflow-wrap: anywhere;
  }
  .subtitle {
    color: var(--text-2);
  }
  .badges {
    display: flex;
    flex-wrap: wrap;
    gap: var(--s-2);
    margin-top: var(--s-3);
  }

  .toc {
    display: flex;
    flex-wrap: wrap;
    gap: var(--s-2);
    margin-bottom: var(--s-4);
  }
  .toc a {
    padding: 4px var(--s-3);
    border-radius: 999px;
    background: var(--surface-2);
    color: var(--text-2);
    font-size: var(--fs-small);
    font-weight: 500;
    text-decoration: none;
  }
  .toc a:hover {
    background: var(--accent-soft);
    color: var(--accent);
  }

  /* masonry: cards flow into as many columns as fit */
  .columns {
    columns: 2 440px;
    column-gap: var(--s-4);
  }
  .block {
    break-inside: avoid;
    margin-bottom: var(--s-4);
    padding: var(--s-4);
    scroll-margin-top: var(--s-4);
  }
  h2 {
    margin-bottom: var(--s-2);
    font-size: var(--fs-label);
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.06em;
    color: var(--text-3);
  }
  dl {
    margin: 0;
  }
  .row {
    display: flex;
    justify-content: space-between;
    gap: var(--s-4);
    padding: 7px 0;
    border-bottom: 1px solid var(--border);
  }
  .row:last-child {
    border-bottom: 0;
  }
  .row.stacked {
    flex-direction: column;
    gap: 2px;
  }
  dt {
    flex: none;
    color: var(--text-2);
  }
  dd {
    margin: 0;
    min-width: 0;
    font-weight: 500;
    text-align: right;
    overflow-wrap: anywhere;
    user-select: text;
  }
  .stacked dd {
    text-align: left;
  }
  pre {
    margin: 0;
    padding: var(--s-3);
    border-radius: var(--radius-sm);
    background: var(--surface-2);
    font: 12px/1.5 ui-monospace, "SF Mono", Menlo, Consolas, monospace;
    font-weight: 400;
    overflow-x: auto;
  }

  .note {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: var(--s-3);
    padding: var(--s-6);
    color: var(--text-2);
    text-align: center;
  }
  .muted {
    color: var(--text-3);
  }
  .error {
    color: var(--danger);
  }

  @media (max-width: 640px) {
    .hero {
      flex-direction: column;
      align-items: flex-start;
    }
  }
</style>
