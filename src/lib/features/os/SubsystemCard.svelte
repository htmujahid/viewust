<script lang="ts">
  import Icon, { type IconName } from "$lib/components/ui/Icon.svelte";

  import type { Fact } from "./subsystems";

  let {
    href = null,
    icon,
    title,
    text,
    facts,
    loading = false,
    error = null,
  }: {
    href?: string | null;
    icon: IconName;
    title: string;
    text: string;
    facts: readonly Fact[];
    loading?: boolean;
    error?: string | null;
  } = $props();
</script>

<section class="card sub">
  {#if href}
    <a class="head" {href}>
      <Icon name={icon} size={18} />
      <b>{title}</b>
      <span class="go" aria-hidden="true">→</span>
    </a>
  {:else}
    <div class="head">
      <Icon name={icon} size={18} />
      <b>{title}</b>
    </div>
  {/if}
  <p class="text">{text}</p>

  {#if error && facts.length === 0}
    <p class="note error">{error}</p>
  {:else if loading && facts.length === 0}
    <p class="note">Reading…</p>
  {:else}
    <dl>
      {#each facts as f (f.label)}
        <div>
          <dt>{f.label}</dt>
          <dd class="tabular {f.tone ?? ''}">{f.value}</dd>
        </div>
      {/each}
    </dl>
  {/if}
</section>

<style>
  .sub {
    display: flex;
    flex-direction: column;
    margin: 0;
    padding: var(--s-4);
  }
  .head {
    display: flex;
    align-items: center;
    gap: var(--s-2);
    color: var(--text);
    text-decoration: none;
  }
  .head b {
    font-weight: 650;
  }
  .head :global(svg) {
    flex: none;
    color: var(--accent);
  }
  .go {
    margin-left: auto;
    color: var(--text-3);
    transition: transform 0.12s ease;
  }
  a.head:hover {
    color: var(--accent);
  }
  a.head:hover .go {
    color: var(--accent);
    transform: translateX(3px);
  }
  .text {
    margin: var(--s-1) 0 var(--s-3);
    color: var(--text-3);
    font-size: var(--fs-label);
  }
  dl {
    display: flex;
    flex-direction: column;
    gap: 6px;
    margin: auto 0 0;
  }
  dl div {
    display: flex;
    align-items: baseline;
    justify-content: space-between;
    gap: var(--s-3);
  }
  dt {
    color: var(--text-2);
    font-size: var(--fs-small);
  }
  dd {
    margin: 0;
    font-family: var(--font-mono);
    font-size: var(--fs-small);
    text-align: right;
  }
  .ok {
    color: var(--ok);
  }
  .warn {
    color: var(--warn);
  }
  .danger {
    color: var(--danger);
  }
  .note {
    color: var(--text-3);
    font-size: var(--fs-small);
  }
  .note.error {
    color: var(--danger);
  }
</style>
