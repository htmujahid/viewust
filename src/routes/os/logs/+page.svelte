<script lang="ts">
  import { onMount } from "svelte";

  import { goto } from "$app/navigation";

  import RescanButton from "$lib/components/ui/RescanButton.svelte";
  import SectionShell from "$lib/components/ui/SectionShell.svelte";
  import StatTiles from "$lib/components/ui/StatTiles.svelte";
  import { osOverview } from "$lib/features/os/overview.svelte";

  // A stable handle: the Loaded instance itself never changes, only its fields do.
  const logs = osOverview.logs;
  const data = $derived(logs.data);

  function onkeydown(e: KeyboardEvent) {
    if (e.key === "Escape") goto("/os");
  }

  onMount(logs.ensure);
</script>

<svelte:window {onkeydown} />

<SectionShell scroll title="Logs">
  {#snippet actions()}
    <RescanButton loading={logs.loading} onclick={logs.load} label />
  {/snippet}

  {#if logs.error && !data}
    <p class="error">Couldn't read the journal: {logs.error}</p>
  {:else if !data}
    <div class="skeleton" style="height: 96px; margin-bottom: 16px"></div>
    <div class="skeleton" style="height: 320px"></div>
  {:else if !data.available}
    <p class="empty">{data.note}</p>
  {:else}
    <StatTiles
      items={[
        { label: "Journal size", value: data.size ?? "—", sub: "on disk" },
        { label: "Boots kept", value: data.boots === null ? "—" : String(data.boots) },
        {
          label: "Errors this boot",
          value: data.truncated ? `over ${data.errors.length}` : String(data.errors.length),
          tone: data.errors.length ? "warn" : "ok",
          sub: data.errors.length ? "worth a look" : "all quiet",
        },
      ]}
    />

    {#if data.note}<p class="hint">{data.note}</p>{/if}

    <section class="card block">
      <h2>Errors this boot</h2>
      {#if data.errors.length}
        {#if data.truncated}<p class="hint">Showing the most recent {data.errors.length}.</p>{/if}
        <pre class="logs">{data.errors.join("\n")}</pre>
      {:else}
        <p class="hint">No error-level messages since this boot.</p>
      {/if}
    </section>
  {/if}
</SectionShell>

<style>
  .block {
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
  .logs {
    max-height: 60vh;
    margin: 0;
    padding: var(--s-3);
    overflow: auto;
    background: var(--bg);
    border: 1px solid var(--border);
    border-left: 2px solid var(--warn);
    color: var(--text-2);
    font: 11.5px/1.5 var(--font-mono);
    white-space: pre-wrap;
    user-select: text;
  }
  .hint {
    margin-bottom: var(--s-2);
    color: var(--text-3);
    font-size: var(--fs-small);
  }
  .empty,
  .error {
    padding: var(--s-6) 0;
    text-align: center;
  }
  .empty {
    color: var(--text-3);
  }
  .error {
    color: var(--danger);
  }
</style>
