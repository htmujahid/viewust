<script lang="ts">
  import { Handle, Position, type NodeProps, type Node } from "@xyflow/svelte";
  import { goto } from "$app/navigation";
  import Handles from "./Handles.svelte";
  import Illustration, { artSize } from "./Illustration.svelte";
  import { deviceUrl, type ComputerData } from "./graph";

  let { id, data, selected }: NodeProps<Node<ComputerData>> = $props();
</script>

<div
  class="node"
  class:selected
  role="button"
  tabindex="-1"
  ondblclick={(e) => {
    e.stopPropagation();
    goto(deviceUrl(id));
  }} style:width="{artSize("computer").width}px" style:height="{artSize("computer").height}px" title={data.name} aria-label={data.name}>
  <Handles outputs />
  <!-- the internet link leaves lower on the right side so it never shares a line with devices -->
  <Handle id="out-r2" type="source" position={Position.Right} style="top: 85%" />
  <Illustration kind="computer" />
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
</style>
