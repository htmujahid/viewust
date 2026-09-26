<script lang="ts">
  /** A row of page links styled as tabs; the one matching `current` is highlighted. */
  let {
    tabs,
    current,
    label,
  }: {
    tabs: readonly { href: string; label: string }[];
    current: string;
    /** Accessible name for the group of links. */
    label: string;
  } = $props();
</script>

<nav aria-label={label}>
  {#each tabs as tab}
    <a href={tab.href} aria-current={current === tab.href ? "page" : undefined}>{tab.label}</a>
  {/each}
</nav>

<style>
  nav {
    display: flex;
    flex-wrap: wrap;
    gap: var(--s-1);
    width: fit-content;
    padding: var(--s-1);
    border-radius: 12px;
    background: var(--surface-2);
  }
  a {
    padding: 6px var(--s-4);
    border-radius: 9px;
    color: var(--text-2);
    font-weight: 500;
    text-decoration: none;
  }
  a:hover {
    color: var(--text);
  }
  a[aria-current="page"] {
    background: var(--surface);
    color: var(--text);
    box-shadow: var(--shadow);
  }
</style>
