<script lang="ts">
  import type { Node, NodeProps } from "@xyflow/svelte";

  import Icon from "$lib/components/ui/Icon.svelte";
  import Illustration from "$lib/illustrations/Illustration.svelte";
  import { artSize } from "$lib/illustrations/sizes";

  import Handles from "./Handles.svelte";
  import type { DeviceData } from "./model";
  import NodeFrame from "./NodeFrame.svelte";

  let { id, data, selected }: NodeProps<Node<DeviceData>> = $props();

  const size = $derived(artSize(data.kind, data.cm));
</script>

<NodeFrame {id} {selected} width={size.width} height={size.height} label={data.tooltip}>
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
</NodeFrame>

<style>
  .caption {
    position: absolute;
    left: 50%;
    bottom: -30px;
    display: inline-flex;
    align-items: center;
    gap: 6px;
    padding: 3px 10px;
    border-radius: 0;
    background: var(--surface);
    box-shadow: 0 0 0 1px var(--border);
    color: var(--text-2);
    font-family: var(--font-mono);
    font-size: 11px;
    font-weight: 600;
    letter-spacing: 0.04em;
    text-transform: uppercase;
    white-space: nowrap;
    transform: translateX(-50%);
  }
  .caption i {
    width: 8px;
    height: 8px;
    border-radius: 0;
  }
  .caption.ok i {
    background: var(--ok);
    box-shadow: 0 0 6px var(--ok);
  }
  .caption.warn i {
    background: var(--warn);
    box-shadow: 0 0 6px var(--warn);
  }
  .caption.danger i {
    background: var(--danger);
    box-shadow: 0 0 6px var(--danger);
  }
  .bars {
    position: absolute;
    bottom: -6px;
    right: -6px;
    display: flex;
    align-items: flex-end;
    gap: 2px;
    padding: 4px 5px;
    border-radius: 0;
    background: var(--surface);
    box-shadow: 0 0 0 1px var(--border);
  }
  .bars i {
    width: 3px;
    border-radius: 0;
    background: var(--surface-2);
  }
  .bars i.on {
    background: var(--ok);
  }
  .radio {
    position: absolute;
    top: -10px;
    right: -10px;
    display: grid;
    place-items: center;
    width: 24px;
    height: 24px;
    border-radius: 0;
    background: var(--accent);
    box-shadow: var(--glow);
    color: var(--accent-ink);
  }
</style>
