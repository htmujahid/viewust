<script lang="ts">
  import type { Snippet } from "svelte";
  import { page } from "$app/state";
  import { activeEntry, SIDEBAR } from "$lib/nav";

  import Icon from "./Icon.svelte";
  import ThemeToggle from "./ThemeToggle.svelte";
  import TitleBar from "./TitleBar.svelte";

  let { children }: { children: Snippet } = $props();
  // Hardware's three pages stand alone; everything the computer runs is behind Software.
  const links = SIDEBAR;
  const active = (href: string) => activeEntry(page.url.pathname) === href;
</script>

<div class="shell">
  <TitleBar />
  <aside class="sidebar">
    <a
      class="brand"
      href="/"
      aria-label="Overview"
      title="Overview"
      aria-current={page.url.pathname === "/" ? "page" : undefined}
    >
      <svg
        viewBox="0 0 24 24"
        width="26"
        height="26"
        fill="none"
        stroke="currentColor"
        stroke-width="1.6"
        stroke-linecap="square"
        stroke-linejoin="miter"
        aria-hidden="true"
      >
        <rect x="8" y="8" width="8" height="8" />
        <rect x="11" y="11" width="2" height="2" fill="currentColor" stroke="none" />
        <path d="M10 8V5H6M16 10h3V6M8 14H5v4M14 16v3h4" />
        <circle cx="4.5" cy="5" r="1.4" fill="var(--bg)" />
        <circle cx="19" cy="4.5" r="1.4" fill="var(--bg)" />
        <circle cx="4.5" cy="19" r="1.4" fill="var(--bg)" />
        <circle cx="19" cy="19" r="1.4" fill="var(--bg)" />
      </svg>
    </a>
    <nav aria-label="Main navigation">
      {#each links as link}
        <a
          href={link.href}
          aria-label={link.label}
          title={link.label}
          aria-current={active(link.href) ? "page" : undefined}
        >
          <Icon name={link.icon} size={20} />
          <span class="code" aria-hidden="true">{link.code}</span>
        </a>
      {/each}
    </nav>
    <div class="appearance"><ThemeToggle /></div>
  </aside>
  <main>{@render children()}</main>
</div>

<style>
  .shell {
    display: grid;
    grid-template-columns: 64px minmax(0, 1fr);
    grid-template-rows: auto minmax(0, 1fr);
    grid-template-areas:
      "titlebar titlebar"
      "sidebar main";
    height: 100vh;
    height: 100dvh;
  }
  .shell > :global(.titlebar) {
    grid-area: titlebar;
  }
  .sidebar {
    grid-area: sidebar;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 28px;
    padding: 16px 8px;
    border-right: 1px solid var(--border);
    background: var(--rail);
    min-height: 0;
    overflow-y: auto;
  }
  .brand {
    display: grid;
    place-items: center;
    width: 36px;
    min-height: 36px;
    color: var(--accent);
    background: var(--accent-soft);
    border: 1px solid var(--accent);
    box-shadow: var(--glow);
    text-shadow: var(--glow-text);
    text-decoration: none;
  }
  .brand[aria-current="page"] {
    box-shadow: var(--glow-strong);
  }
  .brand svg {
    filter: drop-shadow(0 0 3px var(--accent));
  }
  nav {
    display: grid;
    gap: 8px;
  }
  nav a {
    position: relative;
    display: grid;
    place-items: center;
    width: 44px;
    height: 44px;
    border: 1px solid transparent;
    color: var(--text-2);
    text-decoration: none;
  }
  nav a:hover {
    background: var(--surface-2);
    border-color: var(--border-strong);
    color: var(--text);
  }
  nav a[aria-current="page"] {
    color: var(--accent);
    background: var(--accent-soft);
    border-color: var(--border);
    border-left: 3px solid var(--accent);
    box-shadow: var(--glow);
  }
  .code {
    position: absolute;
    right: 3px;
    bottom: 1px;
    color: var(--text-3);
    font: 600 8px var(--font-mono);
    letter-spacing: 0.04em;
  }
  nav a[aria-current="page"] .code {
    color: var(--accent);
  }
  .appearance {
    margin-top: auto;
    padding-top: 16px;
    border-top: 1px solid var(--border);
  }
  main {
    grid-area: main;
    min-width: 0;
    min-height: 0;
    overflow: hidden;
  }
  @media (max-width: 480px) {
    .shell {
      grid-template-columns: 52px minmax(0, 1fr);
    }
    .sidebar {
      padding: 12px 4px;
    }
    nav a {
      width: 40px;
      height: 40px;
    }
  }
</style>
