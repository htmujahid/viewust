<script lang="ts">
  import { fly } from "svelte/transition";
  import { goto } from "$app/navigation";
  import Badge from "$lib/components/ui/Badge.svelte";
  import Illustration from "$lib/illustrations/Illustration.svelte";
  import DetailList from "$lib/components/ui/DetailList.svelte";
  import { groupBySection } from "$lib/utils/details";
  import { deviceUrl, infoRows, type NodeInfo } from "./model";

  let { info, onclose }: { info: NodeInfo; onclose: () => void } = $props();

  const sections = $derived(groupBySection(infoRows(info)));

  function onkeydown(e: KeyboardEvent) {
    if (e.key === "Escape") onclose();
  }
</script>

<svelte:window {onkeydown} />

<aside
  class="card panel"
  transition:fly={{ x: 24, duration: 180 }}
  aria-label="{info.title} details"
>
  <header>
    <div class="art"><Illustration kind={info.kind} /></div>
    <button class="close" onclick={onclose} aria-label="Close details">
      <svg
        width="16"
        height="16"
        viewBox="0 0 24 24"
        fill="none"
        stroke="currentColor"
        stroke-width="2"
        stroke-linecap="round"
      >
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
    <svg
      width="14"
      height="14"
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      stroke-width="2"
      stroke-linecap="round"
      stroke-linejoin="round"><path d="M5 12h14M13 6l6 6-6 6" /></svg
    >
  </button>

  <div class="body">
    {#each sections as section}
      <section>
        <h3>{section.title}</h3>
        <DetailList rows={section.rows} stackAt={48} />
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
    margin: 0;
    border-width: 0 0 0 1px;
    overflow: hidden;
  }

  header {
    position: relative;
    flex: none;
    height: 150px;
    padding: var(--s-4) 48px;
    background: var(--surface-2);
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
    border-radius: 0;
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
    font-family: var(--font-mono);
    font-size: var(--fs-label);
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.1em;
  }
  .kind::before {
    content: "> ";
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
    border-radius: 0;
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
  .empty {
    padding: var(--s-5) 0;
    color: var(--text-3);
    text-align: center;
  }
  @media (max-width: 1050px) {
    .panel {
      position: absolute;
      right: 0;
      top: 0;
      bottom: 0;
      z-index: 10;
      width: min(400px, 100%);
      box-shadow: -12px 0 32px rgb(0 0 0 / 0.45);
    }
  }
</style>
