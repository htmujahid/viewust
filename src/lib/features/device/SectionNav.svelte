<script lang="ts">
  import { sectionSlug, type Section } from "$lib/utils/details";

  /** Pills that jump to each section of the page. */
  let { sections }: { sections: readonly Section[] } = $props();
</script>

<nav aria-label="Sections">
  {#each sections as s}
    <a href="#{sectionSlug(s.title)}">{s.title}</a>
  {/each}
</nav>

<style>
  nav {
    display: flex;
    flex-wrap: wrap;
    gap: var(--s-2);
    margin-bottom: var(--s-4);
  }
  a {
    padding: 4px var(--s-3);
    border-radius: 0;
    background: var(--surface-2);
    border: 1px solid var(--border);
    color: var(--text-2);
    font-family: var(--font-mono);
    font-size: 11px;
    font-weight: 500;
    letter-spacing: 0.04em;
    text-decoration: none;
    text-transform: uppercase;
  }
  a:hover {
    background: var(--accent-soft);
    border-color: var(--accent);
    color: var(--accent);
    box-shadow: var(--glow);
  }
</style>
