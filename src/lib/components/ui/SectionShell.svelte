<script lang="ts">
  import type { Snippet } from "svelte";

  import { page } from "$app/state";

  import Page from "$lib/components/ui/Page.svelte";
  import TabNav from "$lib/components/ui/TabNav.svelte";

  let {
    tabs,
    label,
    actions,
    panel,
    children,
  }: {
    tabs: readonly { href: string; label: string }[];
    label: string;
    actions?: Snippet;
    panel?: Snippet;
    children: Snippet;
  } = $props();
</script>

<div class="layout">
  <div class="content">
    <Page fill>
      <div class="bar">
        <TabNav {tabs} current={page.url.pathname} {label} />
        {#if actions}<div class="actions">{@render actions()}</div>{/if}
      </div>
      {@render children()}
    </Page>
  </div>
  {#if panel}{@render panel()}{/if}
</div>

<style>
  .layout {
    position: relative;
    display: flex;
    height: 100%;
  }
  .content {
    flex: 1;
    min-width: 0;
  }
  .bar {
    display: flex;
    flex-wrap: wrap;
    align-items: flex-end;
    justify-content: space-between;
    gap: var(--s-3);
    margin-bottom: var(--s-4);
  }
  .bar :global(nav) {
    padding: 0;
  }
  .bar :global(nav a) {
    margin-bottom: -1px;
  }
  .actions {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: var(--s-3);
  }
</style>
