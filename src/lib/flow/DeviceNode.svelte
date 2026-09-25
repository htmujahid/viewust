<script lang="ts">
  import { type NodeProps, type Node } from "@xyflow/svelte";
  import Icon from "$lib/components/Icon.svelte";
  import { goto } from "$app/navigation";
  import Handles from "./Handles.svelte";
  import Illustration, { artSize } from "./Illustration.svelte";
  import { deviceUrl, type DeviceData } from "./graph";

  let { id, data, selected }: NodeProps<Node<DeviceData>> = $props();
</script>

<div
  class="node"
  class:selected
  role="button"
  tabindex="-1"
  ondblclick={(e) => {
    e.stopPropagation();
    goto(deviceUrl(id));
  }} style:width="{artSize(data.kind, data.cm).width}px" style:height="{artSize(data.kind, data.cm).height}px" title={data.tooltip} aria-label={data.tooltip}>
  <Handles inputs outputs />
  <Illustration kind={data.kind} cm={data.cm} />
  {#if data.caption}
    <span class="caption {data.caption.tone}"><i></i>{data.caption.text}</span>
  {/if}
  {#if data.signal !== undefined}
    <span class="bars" title="Signal {data.signal}%" aria-hidden="true">
      {#each [20, 40, 60, 80] as step, i}
        <i class:on={data.signal >= step} style:height="{5 + i * 3.5}px"></i>
      {/each}
    </span>
  {/if}
  {#if data.wireless}
    <span class="radio" aria-hidden="true"><Icon name="wireless" size={13} /></span>
  {/if}
</div>

<style>
  .node {
    position: relative;
    cursor: pointer;
  }
  .caption {
    position: absolute;
    left: 50%;
    bottom: -30px;
    display: inline-flex;
    align-items: center;
    gap: 6px;
    padding: 3px 10px;
    border-radius: 999px;
    background: var(--surface);
    box-shadow: 0 0 0 1px var(--border);
    color: var(--text-2);
    font-size: 12px;
    font-weight: 600;
    white-space: nowrap;
    transform: translateX(-50%);
  }
  .caption i {
    width: 8px;
    height: 8px;
    border-radius: 50%;
  }
  .caption.ok i {
    background: var(--ok);
  }
  .caption.warn i {
    background: var(--warn);
  }
  .caption.danger i {
    background: var(--danger);
  }
  .bars {
    position: absolute;
    bottom: -6px;
    right: -6px;
    display: flex;
    align-items: flex-end;
    gap: 2px;
    padding: 4px 5px;
    border-radius: 8px;
    background: var(--surface);
    box-shadow: 0 0 0 1px var(--border);
  }
  .bars i {
    width: 3px;
    border-radius: 1px;
    background: var(--surface-2);
  }
  .bars i.on {
    background: var(--ok);
  }
  .node::after {
    content: "";
    position: absolute;
    inset: -10px;
    border: 2px solid var(--accent);
    border-radius: var(--radius);
    opacity: 0;
    transform: scale(0.96);
    transition: opacity 0.15s, transform 0.15s;
    pointer-events: none;
  }
  .node:hover::after {
    opacity: 0.35;
    transform: none;
  }
  .node.selected::after {
    opacity: 1;
    transform: none;
  }
  .radio {
    position: absolute;
    top: -10px;
    right: -10px;
    display: grid;
    place-items: center;
    width: 24px;
    height: 24px;
    border-radius: 50%;
    background: var(--accent);
    color: #fff;
  }
</style>
