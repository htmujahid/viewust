<script lang="ts">
  import { fly } from "svelte/transition";

  import type { ServiceDetail, ServiceRow } from "$lib/api/types";
  import Badge from "$lib/components/ui/Badge.svelte";
  import DetailList from "$lib/components/ui/DetailList.svelte";
  import { groupBySection } from "$lib/utils/details";

  import { bootOf, statusOf } from "./status";

  let {
    row,
    detail,
    onclose,
  }: {
    row: ServiceRow | undefined;
    detail: ServiceDetail | null;
    onclose: () => void;
  } = $props();

  const sections = $derived(groupBySection(detail?.details ?? []));
  const status = $derived(row ? statusOf(row) : null);
  const boot = $derived(row ? bootOf(row.enabled) : null);
</script>

<aside class="card panel" transition:fly={{ x: 24, duration: 180 }} aria-label="Service details">
  <header>
    <div class="title">
      <h2 class="truncate" title={row?.unit}>{row?.name ?? detail?.unit ?? "Service"}</h2>
      {#if row?.description}<p>{row.description}</p>{/if}
      {#if status && boot}
        <div class="badges">
          <Badge tone={status.tone}>{status.label}</Badge>
          <Badge tone={boot.tone}>At boot: {boot.label}</Badge>
        </div>
      {/if}
    </div>
    <button class="close" onclick={onclose} aria-label="Close">
      <svg
        width="16"
        height="16"
        viewBox="0 0 24 24"
        fill="none"
        stroke="currentColor"
        stroke-width="2"
        stroke-linecap="round"><path d="M18 6 6 18M6 6l12 12" /></svg
      >
    </button>
  </header>

  <div class="body">
    {#if !detail}
      <p class="muted pad">Reading…</p>
    {:else if !detail.found}
      <p class="muted pad">This service no longer exists.</p>
    {:else}
      {#each sections as s}
        <section>
          <h3>{s.title}</h3>
          <DetailList rows={s.rows} stackAt={40} />
        </section>
      {/each}

      <section>
        <h3>Recent log</h3>
        {#if detail.logs.length}
          <pre class="logs">{detail.logs.join("\n")}</pre>
        {:else}
          <p class="muted">{detail.logs_note ?? "No log entries."}</p>
        {/if}
      </section>
    {/if}
  </div>
</aside>

<style>
  .panel {
    display: flex;
    flex: none;
    flex-direction: column;
    align-self: stretch;
    width: 420px;
    margin: 0;
    border-width: 0 0 0 1px;
    overflow: hidden;
  }
  header {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    gap: var(--s-3);
    padding: var(--s-4);
    border-bottom: 1px solid var(--border);
  }
  .title {
    min-width: 0;
  }
  h2 {
    font-size: 17px;
    font-weight: 650;
  }
  .title p {
    color: var(--text-2);
  }
  .badges {
    display: flex;
    flex-wrap: wrap;
    gap: var(--s-2);
    margin-top: var(--s-2);
  }
  .close {
    display: grid;
    place-items: center;
    flex: none;
    width: 28px;
    height: 28px;
    border: 0;
    border-radius: 0;
    background: var(--surface-2);
    color: var(--text-2);
    cursor: pointer;
  }
  .close:hover {
    color: var(--text);
  }
  .body {
    flex: 1;
    overflow-y: auto;
    padding: 0 var(--s-4) var(--s-4);
  }
  section {
    margin-top: var(--s-4);
  }
  h3 {
    margin-bottom: var(--s-2);
    font-size: var(--fs-label);
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.06em;
    color: var(--text-3);
  }
  .muted {
    color: var(--text-3);
  }
  .pad {
    padding: var(--s-5) 0;
    text-align: center;
  }
  .logs {
    max-height: 320px;
    margin: 0;
    padding: var(--s-3);
    overflow: auto;
    background: var(--bg);
    border: 1px solid var(--border);
    border-left: 2px solid var(--accent);
    color: var(--ok);
    font: 11.5px/1.5 var(--font-mono);
    white-space: pre;
    user-select: text;
  }
  @media (max-width: 1050px) {
    .panel {
      position: absolute;
      right: 0;
      top: 0;
      bottom: 0;
      z-index: 10;
      width: min(420px, 100%);
      box-shadow: -12px 0 32px rgb(0 0 0 / 0.45);
    }
  }
</style>
