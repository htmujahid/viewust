<script lang="ts">
  let {
    items,
  }: {
    items: readonly {
      label: string;
      value: string;
      sub?: string;
      tone?: "ok" | "warn" | "danger";
    }[];
  } = $props();
</script>

<section class="tiles">
  {#each items as t (t.label)}
    <div class="card tile">
      <p class="label">{t.label}</p>
      <p class="value tabular {t.tone ?? ''}">
        {t.value}
        {#if t.sub}<small>{t.sub}</small>{/if}
      </p>
    </div>
  {/each}
</section>

<style>
  .tiles {
    display: grid;
    grid-template-columns: repeat(4, minmax(0, 1fr));
    gap: var(--s-4);
    margin-bottom: var(--s-5);
  }
  @media (max-width: 900px) {
    .tiles {
      grid-template-columns: repeat(2, minmax(0, 1fr));
    }
  }
  .tile {
    display: flex;
    flex-direction: column;
    gap: var(--s-2);
    padding: var(--s-4);
  }
  .label {
    color: var(--text-3);
    font-size: var(--fs-label);
    font-weight: 600;
    letter-spacing: 0.06em;
    text-transform: uppercase;
  }
  .value {
    font-size: 22px;
    font-weight: 650;
    letter-spacing: -0.01em;
  }
  .value small {
    color: var(--text-2);
    font-size: var(--fs-small);
    font-weight: 400;
  }
  .ok {
    color: var(--ok);
  }
  .warn {
    color: var(--warn);
  }
  .danger {
    color: var(--danger);
    text-shadow: 0 0 10px color-mix(in srgb, var(--danger) 55%, transparent);
  }
</style>
