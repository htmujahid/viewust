<script lang="ts">
  import { fly } from "svelte/transition";
  import { goto } from "$app/navigation";
  import Badge from "$lib/components/Badge.svelte";
  import Illustration from "./Illustration.svelte";
  import { deviceUrl, type NodeInfo } from "./graph";

  let { info, onclose }: { info: NodeInfo; onclose: () => void } = $props();

  // Rows arrive flat; group by section, keeping the order they were sent in.
  const sections = $derived.by(() => {
    const groups: { title: string; rows: { label: string; value: string }[] }[] = [];
    const rows = info.via
      ? [{ section: "Connection", label: "Connected through", value: info.via }, ...info.details]
      : info.details;
    for (const r of rows) {
      let group = groups.find((g) => g.title === r.section);
      if (!group) groups.push((group = { title: r.section, rows: [] }));
      group.rows.push({ label: r.label, value: r.value });
    }
    return groups;
  });

  function onkeydown(e: KeyboardEvent) {
    if (e.key === "Escape") onclose();
  }
</script>

<svelte:window {onkeydown} />

<aside class="card panel" transition:fly={{ x: 24, duration: 180 }} aria-label="{info.title} details">
  <header>
    <div class="art"><Illustration kind={info.kind} /></div>
    <button class="close" onclick={onclose} aria-label="Close details">
      <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round">
        <path d="M18 6 6 18M6 6l12 12" />
      </svg>
    </button>
  </header>

  <div class="title">
    <p class="kind">{info.label}</p>
    <h2>{info.title}</h2>
    {#if info.subtitle}<p class="subtitle">{info.subtitle}</p>{/if}
    {#if info.connection}
      <div class="badge">
        <Badge tone={info.connection === "Wireless" ? "ok" : "neutral"}>{info.connection}</Badge>
      </div>
    {/if}
  </div>

  <button class="open" onclick={() => goto(deviceUrl(info.id))}>
    {info.id === "computer" ? "Open internal components" : "Open full technical details"}
    <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M5 12h14M13 6l6 6-6 6" /></svg>
  </button>

  <div class="body">
    {#each sections as section}
      <section>
        <h3>{section.title}</h3>
        <dl>
          {#each section.rows as row}
            <div class="row">
              <dt>{row.label}</dt>
              <dd>{row.value}</dd>
            </div>
          {/each}
        </dl>
      </section>
    {:else}
      <p class="empty">No further details are available for this device.</p>
    {/each}
  </div>
</aside>

<style>
  .panel {
    display: flex;
    flex: none;
    flex-direction: column;
    align-self: stretch;
    width: 340px;
    margin: var(--s-4) var(--s-4) var(--s-4) 0;
    overflow: hidden;
  }

  header {
    position: relative;
    flex: none;
    height: 150px;
    padding: var(--s-4) 48px;
    background: radial-gradient(circle at 50% 45%, var(--accent-soft), var(--surface-2) 90%);
    border-bottom: 1px solid var(--border);
  }
  /* The drawing is pinned to this box; its viewBox scales it to fit. */
  .art {
    position: relative;
    width: 100%;
    height: 100%;
  }
  /* Fill the box and let the viewBox scale the drawing to fit inside it. */
  .art :global(svg) {
    position: absolute;
    inset: 0;
    width: 100%;
    height: 100%;
  }
  .close {
    position: absolute;
    top: var(--s-3);
    right: var(--s-3);
    display: grid;
    place-items: center;
    width: 28px;
    height: 28px;
    border: 0;
    border-radius: 50%;
    background: var(--surface);
    color: var(--text-2);
    cursor: pointer;
  }
  .close:hover {
    color: var(--text);
  }

  .title {
    padding: var(--s-4) var(--s-4) var(--s-3);
  }
  .kind {
    color: var(--accent);
    font-size: var(--fs-label);
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.06em;
  }
  h2 {
    margin-top: 2px;
    font-size: 17px;
    font-weight: 650;
    line-height: 1.25;
    overflow-wrap: anywhere;
  }
  .subtitle {
    color: var(--text-2);
  }
  .badge {
    margin-top: var(--s-2);
  }

  .open {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin: 0 var(--s-4) var(--s-3);
    padding: 8px var(--s-3);
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
    background: var(--accent-soft);
    color: var(--accent);
    font-weight: 600;
    cursor: pointer;
  }
  .open:hover {
    border-color: var(--accent);
  }

  .body {
    flex: 1;
    overflow-y: auto;
    padding: 0 var(--s-4) var(--s-4);
  }
  section + section {
    margin-top: var(--s-4);
  }
  h3 {
    margin-bottom: var(--s-1);
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
  dt {
    flex: none;
    color: var(--text-2);
  }
  dd {
    margin: 0;
    min-width: 0;
    text-align: right;
    font-weight: 500;
    overflow-wrap: anywhere;
    user-select: text;
  }
  .empty {
    padding: var(--s-5) 0;
    color: var(--text-3);
    text-align: center;
  }
</style>
