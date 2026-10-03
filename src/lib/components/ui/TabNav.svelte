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
    border-radius: 0;
    background: transparent;
    border-bottom: 1px solid var(--border);
  }
  a {
    padding: 10px var(--s-4);
    font-family: var(--font-mono);
    font-size: 12px;
    border-radius: 0;
    border-bottom: 2px solid transparent;
    color: var(--text-2);
    font-weight: 500;
    text-decoration: none;
    text-transform: uppercase;
    letter-spacing: 0.06em;
  }
  a:hover {
    color: var(--text);
    background: var(--surface-2);
  }
  a[aria-current="page"] {
    background: var(--accent-soft);
    border-bottom-color: var(--accent);
    color: var(--accent);
    text-shadow: var(--glow-text);
    box-shadow: 0 6px 12px -8px var(--accent);
  }
  a[aria-current="page"]::before {
    content: "> ";
  }
</style>
