<script lang="ts">
  import type { Snippet } from "svelte";

  import BackLink from "./BackLink.svelte";
  import ThemeToggle from "./ThemeToggle.svelte";

  /**
   * The bar across the top of every page: where "back" goes, an optional
   * title, the page's own actions, and the theme switch.
   */
  let {
    back,
    backLabel,
    title,
    subtitle,
    actions,
  }: {
    back: string;
    backLabel: string;
    title?: string;
    subtitle?: string;
    actions?: Snippet;
  } = $props();
</script>

<header class="top">
  <BackLink href={back}>{backLabel}</BackLink>
  {#if title}
    <div class="heading">
      <h1>{title}</h1>
      {#if subtitle}<p>{subtitle}</p>{/if}
    </div>
  {/if}
  <div class="spacer"></div>
  {#if actions}{@render actions()}{/if}
  <ThemeToggle />
</header>

<style>
  .top {
    display: flex;
    align-items: center;
    gap: var(--s-3);
    margin-bottom: var(--s-4);
  }
  .heading h1 {
    font-size: 20px;
    font-weight: 650;
  }
  .heading p {
    color: var(--text-2);
    font-size: var(--fs-small);
  }
  .spacer {
    flex: 1;
  }
</style>
