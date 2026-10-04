<script lang="ts">
  import { tick } from "svelte";

  import { menu, type MenuItem } from "$lib/stores/menu.svelte";

  let el = $state<HTMLDivElement | null>(null);
  let left = $state(0);
  let top = $state(0);

  // Place the menu at the pointer, then pull it back inside the window if it would spill out.
  $effect(() => {
    if (!menu.isOpen) return;
    left = menu.x;
    top = menu.y;
    tick().then(() => {
      if (!el) return;
      const { width, height } = el.getBoundingClientRect();
      left = Math.max(4, Math.min(menu.x, window.innerWidth - width - 4));
      top = Math.max(4, Math.min(menu.y, window.innerHeight - height - 4));
      el.querySelector<HTMLButtonElement>("button:not(:disabled)")?.focus();
    });
  });

  function choose(item: MenuItem) {
    if (item.disabled) return;
    menu.close();
    void item.onselect?.();
  }

  function onkeydown(e: KeyboardEvent) {
    if (!menu.isOpen) return;
    if (e.key === "Escape") {
      e.preventDefault();
      e.stopPropagation();
      menu.close();
      return;
    }
    const buttons = [...(el?.querySelectorAll<HTMLButtonElement>("button:not(:disabled)") ?? [])];
    if (!buttons.length) return;
    const at = buttons.indexOf(document.activeElement as HTMLButtonElement);
    const move = (to: number) => {
      e.preventDefault();
      buttons[(to + buttons.length) % buttons.length].focus();
    };
    if (e.key === "ArrowDown") move(at + 1);
    else if (e.key === "ArrowUp") move(at < 0 ? -1 : at - 1);
    else if (e.key === "Home") move(0);
    else if (e.key === "End") move(-1);
    else if (e.key === "Tab") menu.close();
  }

  function outside(e: Event) {
    if (menu.isOpen && el && !el.contains(e.target as Node)) menu.close();
  }
</script>

<svelte:window
  onkeydowncapture={onkeydown}
  onpointerdowncapture={outside}
  onblur={menu.close}
  onresize={menu.close}
  onwheelcapture={outside}
/>

{#if menu.isOpen}
  <div
    class="menu"
    role="menu"
    tabindex="-1"
    bind:this={el}
    style:left="{left}px"
    style:top="{top}px"
    oncontextmenu={(e) => e.preventDefault()}
  >
    {#if menu.title}<p class="title truncate" title={menu.title}>{menu.title}</p>{/if}
    {#each menu.items as item}
      {#if item.separator}
        <hr />
      {:else}
        <button
          role="menuitem"
          class:danger={item.danger}
          disabled={item.disabled}
          onclick={() => choose(item)}
        >
          <span>{item.label}</span>
          {#if item.hint}<small>{item.hint}</small>{/if}
        </button>
      {/if}
    {/each}
  </div>
{/if}

<style>
  .menu {
    position: fixed;
    z-index: 100;
    min-width: 220px;
    max-width: 320px;
    padding: var(--s-1) 0;
    border: 1px solid var(--accent);
    background: var(--surface);
    box-shadow:
      0 12px 32px rgb(0 0 0 / 0.5),
      var(--glow);
  }
  .title {
    padding: 6px var(--s-4) 8px;
    margin-bottom: var(--s-1);
    border-bottom: 1px solid var(--border);
    color: var(--text-3);
    font-family: var(--font-mono);
    font-size: 11px;
    letter-spacing: 0.06em;
    text-transform: uppercase;
  }
  hr {
    margin: var(--s-1) 0;
    border: 0;
    border-top: 1px solid var(--border);
  }
  button {
    display: flex;
    align-items: baseline;
    justify-content: space-between;
    gap: var(--s-4);
    width: 100%;
    padding: 7px var(--s-4);
    border: 0;
    background: transparent;
    color: var(--text);
    font-family: var(--font-ui);
    font-size: var(--fs-small);
    text-align: left;
    cursor: pointer;
  }
  button small {
    color: var(--text-3);
    font-family: var(--font-mono);
    font-size: 10.5px;
  }
  button:hover:not(:disabled),
  button:focus-visible {
    outline: 0;
    background: var(--accent-soft);
    color: var(--accent);
  }
  button.danger {
    color: var(--danger);
  }
  button.danger:hover:not(:disabled),
  button.danger:focus-visible {
    background: var(--danger-soft);
    color: var(--danger);
  }
  button:disabled {
    color: var(--text-3);
    cursor: default;
    opacity: 0.55;
  }
</style>
