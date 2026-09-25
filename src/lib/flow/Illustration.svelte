<script lang="ts" module>
  export type Kind =
    | "computer"
    | "monitor"
    | "keyboard"
    | "mouse"
    | "webcam"
    | "audio"
    | "microphone"
    | "gamepad"
    | "storage"
    | "printer"
    | "hub"
    | "wireless"
    | "phone"
    | "securitykey"
    | "generic"
    // inside the computer
    | "board"
    | "cpu"
    | "ram"
    | "gpu"
    | "nvme"
    | "ssd"
    | "hdd"
    | "psu"
    | "nic"
    | "soundcard"
    // the connection to the internet
    | "router"
    | "internet";

  // Tight bounding box of each drawing (x, y, w, h), and how long its longer
  // side is in real life (cm). Sizes are derived from that, so a mouse looks
  // smaller than a headset, which looks smaller than a monitor.
  const ART: Record<Kind, { box: [number, number, number, number]; cm?: number; px?: number; boost?: number }> = {
    // The computer is the hub everything hangs off, so it is drawn larger than
    // its real-world ratio and always outweighs the monitors around it.
    computer: { box: [48, 2, 64, 97], cm: 45, boost: 1.7 }, // tower height
    monitor: { box: [18, 6, 124, 86], cm: 55 }, // panel width (overridden by EDID)
    keyboard: { box: [8, 22, 144, 58], cm: 44 },
    mouse: { box: [46, 6, 68, 88], cm: 11.5 },
    webcam: { box: [44, 4, 72, 88], cm: 9.5 },
    audio: { box: [22, 8, 116, 76], cm: 8.4 }, // headphones: drawn at half their previous size
    microphone: { box: [48, 4, 64, 88], cm: 9.5 }, // drawn the same height as the webcam
    gamepad: { box: [14, 30, 132, 58], cm: 16 },
    storage: { box: [12, 26, 138, 48], cm: 6.5 },
    printer: { box: [16, 6, 128, 88], cm: 40 },
    hub: { box: [12, 8, 136, 70], cm: 12 },
    wireless: { box: [12, 28, 142, 44], cm: 4.5 },
    phone: { box: [52, 2, 56, 96], cm: 15 },
    securitykey: { box: [18, 28, 124, 44], cm: 6 },
    generic: { box: [28, 28, 122, 44], cm: 8 },
    // Internal parts (the computer's own page) are sized by importance rather
    // than real centimetres: a CPU is tiny in life but is a key part here.
    board: { box: [0, 0, 160, 160], px: 330 },
    cpu: { box: [0, 0, 100, 100], px: 100 },
    ram: { box: [0, 0, 160, 44], px: 150 },
    gpu: { box: [0, 0, 200, 92], px: 250 },
    nvme: { box: [0, 0, 140, 44], px: 130 },
    ssd: { box: [0, 0, 120, 90], px: 115 },
    hdd: { box: [0, 0, 140, 100], px: 130 },
    psu: { box: [0, 0, 150, 110], px: 150 },
    nic: { box: [0, 0, 140, 90], px: 115 },
    soundcard: { box: [0, 0, 140, 90], px: 115 },
    router: { box: [0, 0, 150, 100], px: 130 },
    internet: { box: [18, 6, 132, 82], px: 150 },
  };

  // Sizes follow real dimensions closely. A light compression (GAMMA just under
  // 1) stops the biggest things from dominating, and MIN_PX keeps the tiniest
  // (dongles, USB sticks) visible. The computer stays the largest "thing" on
  // the canvas; peripherals come out at roughly their real fraction of it.
  const REF_CM = 45;
  const REF_PX = 200;
  const GAMMA = 0.85;
  const MIN_PX = 40;

  /** On-screen size of a device's drawing. `cm` overrides its real length. */
  export function artSize(kind: Kind, cm?: number): { width: number; height: number } {
    const { box, cm: defaultCm = 10, px: fixed, boost = 1 } = ART[kind];
    const real = Math.min(Math.max(cm ?? defaultCm, 2), 120);
    const longest = Math.max(box[2], box[3]);
    const px = fixed ?? Math.max(MIN_PX, REF_PX * (real / REF_CM) ** GAMMA) * boost;
    const scale = px / longest;
    return { width: Math.round(box[2] * scale), height: Math.round(box[3] * scale) };
  }
</script>

<script lang="ts">
  let { kind, cm }: { kind: Kind; cm?: number } = $props();

  const size = $derived(artSize(kind, cm));
  const viewBox = $derived(ART[kind].box.join(" "));
</script>

<svg {viewBox} width={size.width} height={size.height} role="img" aria-label={kind}>
  {#if kind === "computer"}
    <!-- desktop tower, front view -->
    <rect class="body" x="50" y="4" width="60" height="90" rx="7" />
    <rect class="dark" x="57" y="11" width="46" height="9" rx="2" />
    <rect class="faint" x="62" y="14.5" width="26" height="2" rx="1" />
    <rect class="dark" x="57" y="24" width="46" height="9" rx="2" />
    <rect class="faint" x="62" y="27.5" width="18" height="2" rx="1" />
    <circle class="accent" cx="80" cy="47" r="6.5" />
    <circle class="glint" cx="80" cy="47" r="2.4" />
    <circle class="led" cx="63" cy="47" r="2" />
    <circle class="faint-fill" cx="97" cy="47" r="2" />
    <rect class="dark" x="63" y="58" width="10" height="3.5" rx="1.5" />
    <rect class="dark" x="87" y="58" width="10" height="3.5" rx="1.5" />
    {#each [0, 1, 2, 3, 4] as i}
      <rect class="faint" x="58" y={68 + i * 4.6} width="44" height="2" rx="1" />
    {/each}
    <rect class="faint-fill" x="56" y="94" width="10" height="3" rx="1.5" />
    <rect class="faint-fill" x="94" y="94" width="10" height="3" rx="1.5" />
  {:else if kind === "monitor"}
    <rect class="dark" x="20" y="8" width="120" height="66" rx="6" />
    <rect class="screen" x="25" y="13" width="110" height="56" rx="2" />
    <rect class="accent" x="31" y="19" width="40" height="6" rx="2" />
    <rect class="faint" x="31" y="30" width="98" height="4" rx="2" />
    <rect class="faint" x="31" y="38" width="70" height="4" rx="2" />
    <rect class="accent soft" x="31" y="47" width="30" height="14" rx="3" />
    <rect class="accent soft" x="67" y="47" width="30" height="14" rx="3" />
    <path class="body" d="M70 74h20l3 10H67z" />
    <rect class="body" x="52" y="84" width="56" height="6" rx="3" />
  {:else if kind === "keyboard"}
    <rect class="body" x="10" y="24" width="140" height="54" rx="8" />
    {#each [0, 1, 2] as row}
      {#each Array(row === 2 ? 9 : 11) as _, i}
        <rect class="key" x={(row === 2 ? 25 : 18 + row * 3) + i * 11.4} y={31 + row * 11} width="9" height="8" rx="2" />
      {/each}
    {/each}
    <rect class="key" x="36" y="64" width="88" height="8" rx="2" />
    <rect class="accent" x="18" y="64" width="14" height="8" rx="2" />
    <rect class="accent" x="128" y="64" width="14" height="8" rx="2" />
  {:else if kind === "mouse"}
    <path class="body" d="M80 8C57 8 48 25 48 46v14c0 20 13 32 32 32s32-12 32-32V46C112 25 103 8 80 8z" />
    <path class="accent soft" d="M80 8C57 8 48 25 48 42h32z" />
    <path class="line" d="M48 42h64M80 8v34" />
    <rect class="accent" x="76" y="18" width="8" height="15" rx="4" />
  {:else if kind === "webcam"}
    <path class="body" d="M66 66h28l8 20H58z" />
    <rect class="body" x="46" y="84" width="68" height="6" rx="3" />
    <circle class="body" cx="80" cy="38" r="32" />
    <circle class="dark" cx="80" cy="38" r="22" />
    <circle class="accent" cx="80" cy="38" r="13" />
    <circle class="glint" cx="75" cy="33" r="4" />
    <circle class="led" cx="80" cy="10" r="2.5" />
  {:else if kind === "audio"}
    <path class="band" d="M34 58V48C34 24 52 12 80 12s46 12 46 36v10" />
    <rect class="accent" x="24" y="46" width="22" height="36" rx="9" />
    <rect class="accent" x="114" y="46" width="22" height="36" rx="9" />
    <rect class="soft-fill" x="40" y="52" width="8" height="24" rx="4" />
    <rect class="soft-fill" x="112" y="52" width="8" height="24" rx="4" />
  {:else if kind === "microphone"}
    <path class="band thin" d="M54 36v8a26 26 0 0 0 52 0v-8" />
    <path class="line" d="M80 70v14" />
    <rect class="body" x="56" y="83" width="48" height="7" rx="3.5" />
    <rect class="body" x="62" y="6" width="36" height="52" rx="18" />
    {#each [16, 23, 30, 44, 51] as y}
      <rect class="faint" x="69" y={y} width="22" height="2" rx="1" />
    {/each}
    <rect class="accent" x="62" y="34" width="36" height="6" />
    <circle class="led" cx="80" cy="37" r="1.6" />
  {:else if kind === "gamepad"}
    <path class="body" d="M40 32C26 32 16 60 18 76c1 9 11 11 18 4l15-14h58l15 14c7 7 17 5 18-4 2-16-8-44-22-44z" />
    <rect class="dark" x="45" y="43" width="8" height="18" rx="2" />
    <rect class="dark" x="40" y="48" width="18" height="8" rx="2" />
    <circle class="accent" cx="113" cy="42" r="4.5" />
    <circle class="faint-fill" cx="123" cy="51" r="4.5" />
    <circle class="faint-fill" cx="103" cy="51" r="4.5" />
    <circle class="faint-fill" cx="113" cy="60" r="4.5" />
    <circle class="dark" cx="66" cy="62" r="7" />
    <circle class="dark" cx="94" cy="62" r="7" />
  {:else if kind === "storage"}
    <rect class="metal" x="14" y="38" width="42" height="24" rx="3" />
    <rect class="dark" x="22" y="45" width="7" height="4" rx="1" />
    <rect class="dark" x="22" y="52" width="7" height="4" rx="1" />
    <rect class="dark" x="36" y="45" width="7" height="4" rx="1" />
    <rect class="dark" x="36" y="52" width="7" height="4" rx="1" />
    <rect class="accent" x="54" y="28" width="94" height="44" rx="12" />
    <rect class="accent soft" x="92" y="28" width="10" height="44" />
    <circle class="screen" cx="134" cy="50" r="5" />
  {:else if kind === "printer"}
    <rect class="paper" x="46" y="8" width="68" height="32" rx="2" />
    <rect class="faint" x="54" y="16" width="40" height="3" rx="1.5" />
    <rect class="faint" x="54" y="24" width="52" height="3" rx="1.5" />
    <rect class="body" x="18" y="36" width="124" height="42" rx="9" />
    <rect class="dark" x="36" y="52" width="88" height="5" rx="2.5" />
    <circle class="led" cx="128" cy="45" r="2.5" />
    <rect class="paper" x="46" y="66" width="68" height="26" rx="2" />
    <rect class="accent" x="54" y="73" width="30" height="3" rx="1.5" />
    <rect class="faint" x="54" y="80" width="46" height="3" rx="1.5" />
  {:else if kind === "hub"}
    <path class="cable" d="M80 42V24c0-9 8-12 22-12" />
    <rect class="body" x="14" y="42" width="132" height="34" rx="10" />
    {#each [0, 1, 2, 3] as i}
      <rect class="dark" x={26 + i * 29} y="54" width="20" height="11" rx="2" />
      <rect class="accent soft" x={30 + i * 29} y="58" width="12" height="3" rx="1" />
    {/each}
    <circle class="led" cx="132" cy="49" r="2.5" />
  {:else if kind === "wireless"}
    <rect class="metal" x="14" y="38" width="42" height="24" rx="3" />
    <rect class="dark" x="22" y="45" width="7" height="4" rx="1" />
    <rect class="dark" x="22" y="52" width="7" height="4" rx="1" />
    <rect class="accent" x="54" y="30" width="80" height="40" rx="12" />
    <path class="glyph" d="M88 41l12 18-6 5V36l6 5-12 18" transform="translate(11 -2) scale(1.0)" />
    <path class="wave" d="M144 40c5 6 5 14 0 20M150 34c9 9 9 23 0 32" />
  {:else if kind === "phone"}
    <rect class="body" x="54" y="4" width="52" height="92" rx="11" />
    <rect class="screen" x="59" y="12" width="42" height="76" rx="5" />
    <rect class="dark" x="71" y="7" width="18" height="3" rx="1.5" />
    <rect class="accent" x="64" y="18" width="14" height="14" rx="4" />
    <rect class="accent soft" x="82" y="18" width="14" height="14" rx="4" />
    <rect class="accent soft" x="64" y="38" width="14" height="14" rx="4" />
    <rect class="accent" x="82" y="38" width="14" height="14" rx="4" />
    <rect class="faint" x="64" y="62" width="32" height="4" rx="2" />
    <rect class="faint" x="64" y="70" width="22" height="4" rx="2" />
  {:else if kind === "securitykey"}
    <rect class="accent" x="20" y="30" width="92" height="40" rx="12" />
    <circle class="screen" cx="38" cy="50" r="5.5" />
    <circle class="screen" cx="80" cy="50" r="12" />
    <circle class="accent" cx="80" cy="50" r="5" />
    <rect class="metal" x="110" y="39" width="30" height="22" rx="3" />
    <rect class="dark" x="118" y="45" width="6" height="4" rx="1" />
    <rect class="dark" x="118" y="52" width="6" height="4" rx="1" />
  {:else if kind === "board"}
    <rect class="pcb" x="4" y="4" width="152" height="152" rx="9" />
    <rect class="dark" x="4" y="32" width="11" height="62" rx="2" />
    <rect class="faint" x="7" y="38" width="5" height="4" rx="1" />
    <rect class="faint" x="7" y="46" width="5" height="4" rx="1" />
    <rect class="faint" x="7" y="54" width="5" height="4" rx="1" />
    <rect class="faint" x="7" y="62" width="5" height="4" rx="1" />
    <rect class="faint" x="7" y="70" width="5" height="4" rx="1" />
    <rect class="faint-fill" x="48" y="12" width="48" height="15" rx="2" />
    <rect class="faint-fill" x="22" y="38" width="17" height="48" rx="2" />
    {#each [0, 1, 2, 3, 4, 5] as i}
      <rect class="body" x={52 + i * 7} y="15" width="3" height="9" rx="1" />
    {/each}
    <rect class="dark" x="46" y="36" width="52" height="52" rx="4" />
    <rect class="screen" x="53" y="43" width="38" height="38" rx="3" />
    <rect class="accent" x="58" y="48" width="12" height="3" rx="1.5" />
    {#each [0, 1, 2, 3] as i}
      <rect class="dark" x={108 + i * 9} y="20" width="6" height="72" rx="2" />
      <rect class="accent soft" x={109 + i * 9} y="22" width="4" height="12" rx="1" />
    {/each}
    <rect class="metal" x="143" y="22" width="10" height="38" rx="2" />
    <rect class="dark" x="95" y="100" width="26" height="26" rx="3" />
    <rect class="accent" x="102" y="107" width="12" height="12" rx="2" />
    <rect class="dark" x="22" y="102" width="64" height="7" rx="2" />
    <rect class="dark" x="22" y="118" width="64" height="7" rx="2" />
    <rect class="dark" x="22" y="134" width="64" height="7" rx="2" />
    <rect class="gold" x="124" y="132" width="26" height="9" rx="2" />
    <circle class="faint-fill" cx="14" cy="14" r="2.6" />
    <circle class="faint-fill" cx="146" cy="14" r="2.6" />
    <circle class="faint-fill" cx="14" cy="146" r="2.6" />
    <circle class="faint-fill" cx="146" cy="146" r="2.6" />
  {:else if kind === "cpu"}
    {#each [0, 1, 2, 3, 4, 5, 6, 7] as i}
      <rect class="gold" x={17 + i * 9.5} y="2" width="5" height="7" rx="1.5" />
      <rect class="gold" x={17 + i * 9.5} y="91" width="5" height="7" rx="1.5" />
      <rect class="gold" x="2" y={17 + i * 9.5} width="7" height="5" rx="1.5" />
      <rect class="gold" x="91" y={17 + i * 9.5} width="7" height="5" rx="1.5" />
    {/each}
    <rect class="body" x="8" y="8" width="84" height="84" rx="10" />
    <rect class="metal" x="21" y="21" width="58" height="58" rx="6" />
    <rect class="faint" x="30" y="33" width="40" height="4" rx="2" />
    <rect class="faint" x="30" y="43" width="28" height="4" rx="2" />
    <rect class="accent" x="30" y="57" width="18" height="10" rx="3" />
    <path class="accent" d="M12 78v10h10z" />
  {:else if kind === "ram"}
    <rect class="pcb" x="2" y="4" width="156" height="34" rx="3" />
    <rect class="accent" x="2" y="4" width="156" height="25" rx="4" />
    {#each [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11] as i}
      <rect class="soft-fill" x={10 + i * 12.5} y="8" width="2.4" height="17" rx="1.2" />
    {/each}
    {#each Array(24) as _, i}
      <rect class="gold" x={5 + i * 6.3} y="31" width="4" height="7" rx="1" />
    {/each}
    <rect class="screen" x="77" y="30" width="4" height="9" />
  {:else if kind === "gpu"}
    <rect class="metal" x="0" y="6" width="7" height="82" rx="2" />
    <rect class="dark" x="6" y="6" width="188" height="64" rx="9" />
    <rect class="accent" x="16" y="9" width="170" height="2.4" rx="1.2" />
    {#each [58, 142] as cx}
      <circle class="body" cx={cx} cy="40" r="26" />
      <circle class="dark" cx={cx} cy="40" r="22" />
      {#each [0, 1, 2, 3, 4, 5, 6] as i}
        <ellipse class="faint-fill" cx="0" cy="-11" rx="5.5" ry="11" transform="translate({cx} 40) rotate({i * 51.4})" />
      {/each}
      <circle class="accent" cx={cx} cy="40" r="6" />
    {/each}
    <rect class="body" x="150" y="0" width="34" height="8" rx="2" />
    <rect class="faint" x="84" y="30" width="30" height="2" rx="1" />
    <rect class="faint" x="84" y="36" width="30" height="2" rx="1" />
    <rect class="faint" x="84" y="42" width="30" height="2" rx="1" />
    <rect class="pcb" x="14" y="70" width="170" height="10" />
    {#each Array(18) as _, i}
      <rect class="gold" x={22 + i * 3.6} y="80" width="2.4" height="10" />
    {/each}
    {#each Array(10) as _, i}
      <rect class="gold" x={92 + i * 3.6} y="80" width="2.4" height="10" />
    {/each}
  {:else if kind === "nvme"}
    <rect class="pcb" x="22" y="4" width="116" height="36" rx="3" />
    <rect class="pcb" x="2" y="8" width="22" height="28" />
    {#each Array(11) as _, i}
      <rect class="gold" x={4 + i * 1.9} y="9" width="1.3" height="26" />
    {/each}
    <rect class="screen" x="14" y="7" width="3" height="30" />
    <rect class="dark" x="30" y="10" width="26" height="24" rx="2" />
    <rect class="accent" x="62" y="12" width="20" height="20" rx="2" />
    <rect class="dark" x="88" y="10" width="26" height="24" rx="2" />
    <circle class="screen" cx="134" cy="22" r="4" />
    <rect class="faint" x="34" y="15" width="14" height="2" rx="1" />
  {:else if kind === "ssd"}
    <rect class="metal" x="4" y="6" width="112" height="78" rx="7" />
    <rect class="accent" x="14" y="16" width="72" height="44" rx="4" />
    <rect class="soft-fill" x="21" y="24" width="38" height="4" rx="2" />
    <rect class="soft-fill" x="21" y="33" width="26" height="4" rx="2" />
    <rect class="screen" x="21" y="43" width="14" height="10" rx="2" />
    <rect class="dark" x="90" y="68" width="24" height="12" rx="2" />
    {#each [0, 1, 2, 3, 4, 5] as i}
      <rect class="gold" x={93 + i * 3.4} y="72" width="2" height="5" />
    {/each}
    <circle class="faint-fill" cx="12" cy="14" r="2.4" />
    <circle class="faint-fill" cx="108" cy="14" r="2.4" />
    <circle class="faint-fill" cx="12" cy="76" r="2.4" />
  {:else if kind === "hdd"}
    <rect class="metal" x="4" y="4" width="132" height="92" rx="7" />
    <circle class="body" cx="62" cy="50" r="38" />
    <circle class="screen" cx="62" cy="50" r="30" />
    <circle class="line" cx="62" cy="50" r="22" />
    <circle class="line" cx="62" cy="50" r="14" />
    <circle class="accent" cx="62" cy="50" r="7" />
    <circle class="soft-fill" cx="62" cy="50" r="2.5" />
    <path class="band thin" d="M118 18 L84 38" />
    <circle class="dark" cx="118" cy="18" r="5" />
    <rect class="accent soft" x="104" y="60" width="24" height="14" rx="3" />
    <rect class="dark" x="104" y="82" width="26" height="9" rx="2" />
    {#each [0, 1, 2, 3, 4] as i}
      <rect class="gold" x={107 + i * 4.4} y="85" width="2.4" height="4" />
    {/each}
  {:else if kind === "psu"}
    <path class="cable" d="M138 70c8 0 8-8 8-16M138 80c10 0 10-14 10-24" />
    <path class="line" d="M138 90c12 0 12-22 12-40" />
    <rect class="body" x="6" y="12" width="132" height="90" rx="9" />
    <circle class="dark" cx="54" cy="57" r="36" />
    {#each [10, 20, 30] as r}
      <circle class="line" cx="54" cy="57" r={r} />
    {/each}
    {#each [0, 1, 2, 3, 4, 5] as i}
      <rect class="faint-fill" x="52.5" y="25" width="3" height="64" rx="1.5" transform="rotate({i * 30} 54 57)" />
    {/each}
    <circle class="accent" cx="54" cy="57" r="8" />
    <rect class="dark" x="104" y="24" width="26" height="18" rx="3" />
    <rect class="accent" x="112" y="28" width="10" height="10" rx="2" />
    <rect class="accent soft" x="102" y="52" width="30" height="5" rx="2" />
    <rect class="faint" x="102" y="63" width="30" height="3" rx="1.5" />
    <rect class="faint" x="102" y="72" width="22" height="3" rx="1.5" />
  {:else if kind === "nic"}
    <rect class="metal" x="2" y="4" width="16" height="78" rx="2" />
    <rect class="dark" x="5" y="28" width="11" height="14" rx="1.5" />
    <circle class="led" cx="8" cy="34" r="1.2" />
    <rect class="pcb" x="18" y="8" width="114" height="64" rx="4" />
    <rect class="dark" x="54" y="22" width="30" height="30" rx="3" />
    <rect class="accent" x="62" y="30" width="14" height="14" rx="2" />
    {#each [0, 1, 2, 3, 4] as i}
      <circle class="faint-fill" cx={30 + i * 6} cy="20" r="2.2" />
    {/each}
    <rect class="faint" x="96" y="20" width="26" height="3" rx="1.5" />
    <rect class="faint" x="96" y="28" width="20" height="3" rx="1.5" />
    {#each Array(16) as _, i}
      <rect class="gold" x={34 + i * 3.6} y="72" width="2.4" height="10" />
    {/each}
  {:else if kind === "soundcard"}
    <rect class="metal" x="2" y="4" width="16" height="78" rx="2" />
    <circle class="ok-fill" cx="10" cy="20" r="4" />
    <circle class="accent" cx="10" cy="34" r="4" />
    <circle class="gold" cx="10" cy="48" r="4" />
    <circle class="faint-fill" cx="10" cy="62" r="4" />
    <rect class="pcb" x="18" y="8" width="114" height="64" rx="4" />
    <rect class="dark" x="52" y="24" width="32" height="26" rx="3" />
    <rect class="accent" x="60" y="31" width="16" height="12" rx="2" />
    {#each [0, 1, 2] as i}
      <circle class="faint-fill" cx={98 + i * 10} cy="22" r="3.4" />
    {/each}
    <rect class="faint" x="28" y="56" width="50" height="3" rx="1.5" />
    {#each Array(16) as _, i}
      <rect class="gold" x={34 + i * 3.6} y="72" width="2.4" height="10" />
    {/each}
  {:else if kind === "router"}
    <path class="cable" d="M34 40 L24 8" />
    <path class="cable" d="M116 40 L126 8" />
    <circle class="accent" cx="24" cy="8" r="4" />
    <circle class="accent" cx="126" cy="8" r="4" />
    <path class="wave" d="M60 30c8-9 22-9 30 0M50 20c14-16 36-16 50 0" />
    <rect class="body" x="6" y="40" width="138" height="48" rx="9" />
    {#each [0, 1, 2, 3] as i}
      <rect class="dark" x={20 + i * 22} y="66" width="16" height="12" rx="2" />
      <rect class="gold" x={24 + i * 22} y="70" width="8" height="3" rx="1" />
    {/each}
    <circle class="led" cx="24" cy="52" r="3" />
    <circle class="led" cx="36" cy="52" r="3" />
    <circle class="faint-fill" cx="48" cy="52" r="3" />
    <rect class="faint" x="100" y="49" width="30" height="4" rx="2" />
  {:else if kind === "internet"}
    <path class="cloud" d="M46 84h74c16 0 28-11 28-25 0-13-10-23-23-24-3-16-17-27-33-27-14 0-26 8-31 20-14 1-24 11-24 24 0 7 3 14 9 18 0 0 0 14 0 14z" transform="translate(-4 2)" />
    <circle class="globe" cx="82" cy="52" r="20" />
    <ellipse class="globe" cx="82" cy="52" rx="8" ry="20" />
    <path class="globe" d="M62 52h40M66 41h32M66 63h32" />
  {:else}
    <path class="cable" d="M112 50h36" />
    <rect class="body" x="30" y="30" width="84" height="40" rx="9" />
    <rect class="dark" x="40" y="42" width="48" height="16" rx="3" />
    <rect class="accent" x="46" y="47" width="8" height="6" rx="1" />
    <rect class="accent" x="60" y="47" width="8" height="6" rx="1" />
    <circle class="led" cx="102" cy="40" r="2.5" />
  {/if}
</svg>

<style>
  svg {
    display: block;
    overflow: visible;
  }
  svg :global(*) {
    stroke-linecap: round;
    stroke-linejoin: round;
  }
  .body {
    fill: var(--surface);
    stroke: var(--text-3);
    stroke-width: 1.6;
  }
  .paper {
    fill: var(--surface);
    stroke: var(--text-3);
    stroke-width: 1.6;
  }
  .key {
    fill: var(--surface-2);
    stroke: var(--text-3);
    stroke-width: 1;
  }
  .dark {
    fill: var(--device-dark);
    stroke: var(--text-3);
    stroke-width: 1.4;
  }
  .screen {
    fill: var(--accent-soft);
  }
  .accent {
    fill: var(--accent);
  }
  .accent.soft {
    fill: var(--accent);
    opacity: 0.45;
  }
  .soft-fill {
    fill: #fff;
    opacity: 0.35;
  }
  .faint {
    fill: var(--text-3);
    opacity: 0.45;
  }
  .faint-fill {
    fill: var(--text-3);
    opacity: 0.7;
  }
  .metal {
    fill: var(--surface-2);
    stroke: var(--text-3);
    stroke-width: 1.6;
  }
  .pcb {
    fill: var(--surface-2);
    stroke: var(--text-3);
    stroke-width: 1.6;
  }
  .gold {
    fill: var(--warn);
  }
  .ok-fill {
    fill: var(--ok);
  }
  .cloud {
    fill: var(--surface);
    stroke: var(--text-3);
    stroke-width: 1.8;
  }
  .globe {
    fill: none;
    stroke: var(--accent);
    stroke-width: 2.4;
  }
  .glint {
    fill: #fff;
    opacity: 0.7;
  }
  .led {
    fill: var(--ok);
  }
  .line {
    fill: none;
    stroke: var(--text-3);
    stroke-width: 1.6;
  }
  .band {
    fill: none;
    stroke: var(--text-3);
    stroke-width: 7;
  }
  .band.thin {
    stroke-width: 2.5;
  }
  .cable {
    fill: none;
    stroke: var(--text-3);
    stroke-width: 5;
  }
  .glyph {
    fill: none;
    stroke: #fff;
    stroke-width: 3;
  }
  .wave {
    fill: none;
    stroke: var(--accent);
    stroke-width: 3;
    opacity: 0.6;
  }
</style>
