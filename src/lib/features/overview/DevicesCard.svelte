<script lang="ts">
  import type { HardwareInfo } from "$lib/api/types";
  import Badge from "$lib/components/ui/Badge.svelte";

  import { displaySummary, internetSummary, peripheralKinds } from "./summary";

  let { info }: { info: HardwareInfo } = $props();

  const internet = $derived(internetSummary(info));
  const displays = $derived(displaySummary(info));
  const kinds = $derived(peripheralKinds(info.peripherals));
</script>

<section class="card block">
  <h2>Connected</h2>
  <dl>
    {#if internet}
      <div>
        <dt>Internet</dt>
        <dd><Badge tone={internet.ok ? "ok" : "warn"}>{internet.text}</Badge></dd>
      </div>
    {/if}
    <div>
      <dt>Displays</dt>
      <dd>
        {#if displays.length}
          {#each displays as d}<span>{d}</span>{/each}
        {:else}
          <span class="muted">None detected</span>
        {/if}
      </dd>
    </div>
    <div>
      <dt>External devices</dt>
      <dd>
        {#if kinds.length}
          {#each kinds as k}<span>{k}</span>{/each}
        {:else}
          <span class="muted">Nothing plugged in</span>
        {/if}
      </dd>
    </div>
  </dl>
  <a class="more" href="/devices">Open the device map →</a>
</section>

<style>
  .block {
    padding: var(--s-5);
  }
  h2 {
    margin-bottom: var(--s-2);
    color: var(--text-2);
    font-size: var(--fs-label);
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.08em;
  }
  h2::before {
    content: "// ";
    color: var(--accent);
  }
  dl {
    margin: 0;
  }
  dl > div {
    display: flex;
    justify-content: space-between;
    gap: var(--s-4);
    padding: var(--s-2) 0;
    border-bottom: 1px solid var(--border);
  }
  dt {
    flex: none;
    color: var(--text-2);
    font-family: var(--font-mono);
    font-size: var(--fs-small);
  }
  dd {
    display: flex;
    flex-direction: column;
    align-items: flex-end;
    gap: 2px;
    margin: 0;
    min-width: 0;
    font-family: var(--font-mono);
    font-size: var(--fs-small);
    text-align: right;
    overflow-wrap: anywhere;
  }
  .muted {
    color: var(--text-3);
  }
  .more {
    display: inline-block;
    margin-top: var(--s-3);
    color: var(--accent);
    font-family: var(--font-mono);
    font-size: 11px;
    font-weight: 600;
    letter-spacing: 0.04em;
    text-decoration: none;
    text-transform: uppercase;
  }
  .more:hover {
    text-shadow: var(--glow-text);
  }
</style>
