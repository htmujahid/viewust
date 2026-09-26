<script lang="ts">
  import { Handle, Position, type Node, type NodeProps } from "@xyflow/svelte";

  import Illustration from "$lib/illustrations/Illustration.svelte";
  import { artSize } from "$lib/illustrations/sizes";

  import Handles from "./Handles.svelte";
  import type { ComputerData } from "./model";
  import NodeFrame from "./NodeFrame.svelte";

  let { id, data, selected }: NodeProps<Node<ComputerData>> = $props();

  const size = artSize("computer");
</script>

<NodeFrame {id} {selected} width={size.width} height={size.height} label={data.name}>
  <Handles outputs />
  <!-- the internet link leaves lower on the right side so it never shares a line with devices -->
  <Handle id="out-r2" type="source" position={Position.Right} style="top: 85%" />
  <Illustration kind="computer" />
</NodeFrame>
