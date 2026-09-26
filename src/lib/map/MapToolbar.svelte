<script lang="ts">
  import type { Snippet } from "svelte";

  import BackLink from "$lib/components/ui/BackLink.svelte";
  import NavLinks from "$lib/components/ui/NavLinks.svelte";
  import ThemeToggle from "$lib/components/ui/ThemeToggle.svelte";

  /** The floating card at the top-left of a diagram. */
  let {
    title,
    subtitle,
    back,
    notice,
    children,
  }: {
    title: string;
    subtitle: string;
    /** Where the optional back button goes. */
    back?: { href: string; label: string };
    /** A message shown just under the card (e.g. a failed permission request). */
    notice?: string | null;
    /** Page-specific actions, between the title and the shared links. */
    children?: Snippet;
  } = $props();
</script>

<div class="card toolbar">
  {#if back}<BackLink href={back.href}>{back.label}</BackLink>{/if}
  <div>
    <h1>{title}</h1>
    <p>{subtitle}</p>
  </div>
  {#if children}{@render children()}{/if}
  <NavLinks />
  <ThemeToggle />
</div>
{#if notice}<p class="notice">{notice}</p>{/if}

<style>
  .toolbar {
    display: flex;
    align-items: center;
    gap: var(--s-4);
    padding: var(--s-3) var(--s-4);
  }
  h1 {
    font-size: 16px;
    font-weight: 650;
  }
  p {
    color: var(--text-2);
    font-size: var(--fs-small);
  }
  .notice {
    max-width: 420px;
    margin-top: var(--s-2);
    padding: var(--s-2) var(--s-3);
    border-radius: var(--radius-sm);
    background: var(--warn-soft);
    color: var(--warn);
  }
</style>
