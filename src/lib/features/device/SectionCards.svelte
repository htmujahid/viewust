<script lang="ts">
  import DetailList from "$lib/components/ui/DetailList.svelte";
  import { sectionSlug, type Section } from "$lib/utils/details";

  let {
    sections,
    pending,
    error,
  }: {
    sections: readonly Section[];
    /** The deep report is still being read. */
    pending: boolean;
    error: string | null;
  } = $props();
</script>

<!-- masonry: cards flow into as many columns as fit -->
<div class="columns">
  {#each sections as s (s.title)}
    <section class="card block" id={sectionSlug(s.title)}>
      <h2>{s.title}</h2>
      <DetailList rows={s.rows} stackAt={64} />
    </section>
  {/each}

  {#if pending}
    <section class="card block">
      <h2>Technical details</h2>
      <p class="muted">Reading the device…</p>
    </section>
  {/if}
  {#if error}
    <section class="card block">
      <h2>Technical details</h2>
      <p class="error">Couldn't read them: {error}</p>
    </section>
  {/if}
</div>

<style>
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
  .muted {
    color: var(--text-3);
  }
  .error {
    color: var(--danger);
  }
</style>
