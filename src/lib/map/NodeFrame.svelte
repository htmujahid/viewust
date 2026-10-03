<script lang="ts">
  import type { Snippet } from "svelte";
  import { goto } from "$app/navigation";

  import { deviceUrl } from "./model";

  let {
    id,
    selected,
    width,
    height,
    label,
    children,
  }: {
    id: string;
    selected: boolean;
    width: number;
    height: number;
    label: string;
    children: Snippet;
  } = $props();
</script>

<div
  class="node"
  class:selected
  role="button"
  tabindex="-1"
  style:width="{width}px"
  style:height="{height}px"
  title={label}
  aria-label={label}
  ondblclick={(e) => {
    e.stopPropagation();
    goto(deviceUrl(id));
  }}
>
  {@render children()}
</div>

<style>
  .node {
    position: relative;
    cursor: pointer;
  }
  .node::after {
    content: "";
    position: absolute;
    inset: -10px;
    border: 2px solid var(--accent);
    border-radius: 0;
    opacity: 0;
    transform: scale(0.96);
    transition:
      opacity 0.15s,
      transform 0.15s;
    pointer-events: none;
  }
  .node:hover::after {
    opacity: 0.35;
    transform: none;
  }
  .node.selected::after {
    opacity: 1;
    transform: none;
    box-shadow: var(--glow);
  }
</style>
