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
    border-radius: 999px;
    background: var(--surface-2);
    color: var(--text-2);
    font-size: var(--fs-small);
    font-weight: 500;
    text-decoration: none;
  }
  a:hover {
    background: var(--accent-soft);
    color: var(--accent);
  }
</style>
