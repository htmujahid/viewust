<script lang="ts">
  import type { Disk, DiskKind, DiskVolume } from "$lib/api/types";
  import Badge from "$lib/components/ui/Badge.svelte";
  import Icon from "$lib/components/ui/Icon.svelte";
  import Meter from "$lib/components/ui/Meter.svelte";
  import { formatBytes, formatCount } from "$lib/utils/format";

  import { allVolumes, isBrowsable, usedPercent, type TreeRow } from "./tree";

  let {
    rows,
    ontoggle,
    onmount,
    oncontext,
  }: {
    rows: readonly TreeRow[];
    ontoggle: (row: TreeRow) => void;
    onmount: (volume: DiskVolume) => void;
    oncontext: (row: TreeRow, event: MouseEvent) => void;
  } = $props();

  const INDENT = 20;

  const KIND: Record<DiskKind, string> = {
    nvme: "NVMe SSD",
    ssd: "SSD",
    hdd: "Hard disk",
    usb: "USB drive",
    optical: "Optical drive",
    network: "Network",
  };
  const LAYER: Record<string, string> = {
    crypto_LUKS: "Encrypted",
    LVM2_member: "LVM",
    linux_raid_member: "RAID",
    zfs_member: "ZFS pool",
  };

  const focusable = (r: TreeRow) => r.type === "disk" || r.type === "volume" || r.type === "entry";
  const canToggle = (r: TreeRow) =>
    r.type === "disk" ||
    (r.type === "volume" && r.expandable) ||
    (r.type === "entry" && r.expandable);
  const sharePercent = (share: number) => `${(share * 100).toFixed(share >= 0.1 ? 0 : 1)}%`;

  /** Used space on a drive, from the filesystems mounted on it. */
  function diskPercent(d: Disk): number | null {
    const mounted = allVolumes(d.volumes).filter(isBrowsable);
    if (!mounted.length || d.size <= 0) return null;
    return (mounted.reduce((n, v) => n + (v.used ?? 0), 0) / d.size) * 100;
  }

  const diskIcon = (k: DiskKind) => (k === "usb" ? "usb" : k === "network" ? "network" : "storage");

  // Arrow keys walk the tree the way a file manager's sidebar does.
  function onkeydown(e: KeyboardEvent) {
    const item = (e.target as HTMLElement).closest<HTMLElement>("[data-key]");
    if (!item) return;
    const items = [...(e.currentTarget as HTMLElement).querySelectorAll<HTMLElement>("[data-key]")];
    const at = items.indexOf(item);
    const row = rows.find((r) => r.key === item.dataset.key);
    const go = (i: number) => {
      e.preventDefault();
      items[Math.max(0, Math.min(items.length - 1, i))]?.focus();
    };
    if (e.key === "ArrowDown") go(at + 1);
    else if (e.key === "ArrowUp") go(at - 1);
    else if (e.key === "Home") go(0);
    else if (e.key === "End") go(items.length - 1);
    else if (e.key === "ArrowRight" && row && canToggle(row)) {
      e.preventDefault();
      if (item.getAttribute("aria-expanded") === "false") ontoggle(row);
      else go(at + 1);
    } else if (e.key === "ArrowLeft") {
      e.preventDefault();
      if (row && item.getAttribute("aria-expanded") === "true") ontoggle(row);
      else {
        const level = Number(item.getAttribute("aria-level"));
        for (let i = at - 1; i >= 0; i--) {
          if (Number(items[i].getAttribute("aria-level")) < level) return go(i);
        }
      }
    } else if ((e.key === "Enter" || e.key === " ") && row && canToggle(row)) {
      e.preventDefault();
      ontoggle(row);
    }
  }
</script>

<div class="card tree" role="tree" aria-label="Disk usage" tabindex="-1" {onkeydown}>
  <div class="head" aria-hidden="true">
    <span>Name</span><span>Share</span><span class="r">Size</span><span class="r">%</span><span
      class="r">Files</span
    >
  </div>

  {#each rows as r (r.key)}
    {#if focusable(r)}
      {@const toggles = canToggle(r)}
      <!-- Arrow keys, Enter and Space are handled once on the tree (see onkeydown). -->
      <!-- svelte-ignore a11y_click_events_have_key_events -->
      <div
        class="row"
        class:disk={r.type === "disk"}
        class:volume={r.type === "volume"}
        role="treeitem"
        aria-level={r.depth + 1}
        aria-expanded={toggles ? (r as { expanded: boolean }).expanded : undefined}
        aria-selected="false"
        tabindex="0"
        data-key={r.key}
        onclick={() => toggles && ontoggle(r)}
        oncontextmenu={(e) => oncontext(r, e)}
      >
        <span class="name">
          <span class="indent" style:width="{r.depth * INDENT}px"></span>
          <span
            class="chev"
            class:turn={(r as { expanded: boolean }).expanded}
            class:none={!toggles}
          >
            <Icon name="chevron" size={14} />
          </span>

          {#if r.type === "disk"}
            <Icon name={diskIcon(r.disk.kind)} size={18} />
            <span class="text">
              <b class="truncate">{r.disk.model ?? r.disk.name}</b>
              <small class="truncate muted">
                {r.disk.kind === "network"
                  ? "Network shares"
                  : `${r.disk.path} · ${KIND[r.disk.kind]}`}{r.disk.removable ? " · removable" : ""}
              </small>
            </span>
          {:else if r.type === "volume"}
            {@const v = r.volume}
            <Icon name="storage" size={16} />
            <span class="text">
              <b class="truncate" title={v.mount ?? v.path}>{v.label ?? v.name}</b>
              <small class="truncate muted">
                {v.path.startsWith("/dev/") ? `${v.path} · ` : ""}{v.fstype ??
                  "no filesystem"}{v.mount && v.mount !== v.path ? ` · at ${v.mount}` : ""}
              </small>
            </span>
            {#if v.mountable}
              <Badge tone="warn">Not mounted</Badge>
              <button
                class="btn mini"
                disabled={r.mounting}
                onclick={(e) => {
                  e.stopPropagation();
                  onmount(v);
                }}
              >
                {r.mounting ? "Mounting…" : "Mount"}
              </button>
            {:else if v.role === "swap"}
              <Badge>Swap</Badge>
            {:else if v.role === "empty"}
              <Badge>No filesystem</Badge>
            {:else if v.role === "container"}
              <Badge>{LAYER[v.fstype ?? ""] ?? "Container"}</Badge>
            {/if}
          {:else}
            <Icon name={r.entry.kind === "dir" ? "folder" : "file"} size={16} />
            <span class="text"
              ><span class="truncate" title={r.entry.path}>{r.entry.name}</span></span
            >
            {#if r.entry.mount}<Badge>Another filesystem</Badge>{/if}
            {#if r.entry.unreadable}<Badge tone="warn">No access</Badge>{/if}
            {#if r.entry.kind === "link"}<Badge>Link</Badge>{/if}
          {/if}

          {#if (r.type === "volume" || r.type === "entry") && r.loading}
            <span class="spinner" role="status" aria-label="Measuring"></span>
          {/if}
        </span>

        {#if r.type === "disk"}
          {@const pct = diskPercent(r.disk)}
          <span class="bar"
            >{#if pct !== null}<Meter value={pct} height={6} />{/if}</span
          >
          <span class="r tabular">{formatBytes(r.disk.size)}</span>
          <span class="r tabular muted">{pct === null ? "" : `${pct.toFixed(0)}%`}</span>
          <span></span>
        {:else if r.type === "volume"}
          {@const pct = usedPercent(r.volume)}
          <span class="bar"
            >{#if pct !== null}<Meter value={pct} height={6} />{/if}</span
          >
          <span class="r tabular" class:muted={r.volume.used === null}>
            {formatBytes(r.volume.used ?? r.volume.size)}
          </span>
          <span class="r tabular muted">{pct === null ? "" : `${pct.toFixed(0)}%`}</span>
          <span></span>
        {:else}
          <span class="bar">
            {#if !r.entry.mount}<Meter value={r.share * 100} tone="accent" height={6} />{/if}
          </span>
          <span class="r tabular">{r.entry.mount ? "—" : formatBytes(r.entry.size)}</span>
          <span class="r tabular muted">{r.entry.mount ? "" : sharePercent(r.share)}</span>
          <span class="r tabular muted">
            {r.entry.kind === "dir" && !r.entry.mount ? formatCount(r.entry.files) : ""}
          </span>
        {/if}
      </div>
    {:else if r.type === "more"}
      <div class="row quiet">
        <span class="name">
          <span class="indent" style:width="{r.depth * INDENT}px"></span>
          <span class="chev none"></span>
          <span class="muted">{formatCount(r.count)} smaller items</span>
        </span>
        <span></span>
        <span class="r tabular muted">{formatBytes(r.size)}</span>
        <span></span><span></span>
      </div>
    {:else if r.type === "note"}
      <div class="note {r.tone}" style:padding-left="calc(var(--s-4) + {r.depth * INDENT + 22}px)">
        {r.text}
      </div>
    {/if}
  {/each}
</div>

<style>
  .tree {
    margin: 0;
    outline: 0;
  }
  .head,
  .row {
    display: grid;
    grid-template-columns: minmax(260px, 1fr) minmax(120px, 220px) 96px 56px 72px;
    align-items: center;
    column-gap: var(--s-4);
    padding: 0 var(--s-4);
  }
  .head {
    position: sticky;
    top: 0;
    z-index: 1;
    padding-top: 12px;
    padding-bottom: 12px;
    border-bottom: 1px solid var(--accent);
    background: var(--surface);
    color: var(--text-2);
    font-family: var(--font-mono);
    font-size: 11px;
    font-weight: 600;
    letter-spacing: 0.08em;
    text-transform: uppercase;
  }
  .r {
    text-align: right;
  }
  .row {
    min-height: 38px;
    border-bottom: 1px solid var(--border);
    font-family: var(--font-mono);
    font-size: var(--fs-small);
    cursor: default;
  }
  .row[role="treeitem"] {
    cursor: pointer;
  }
  .row:hover {
    background: var(--surface-2);
  }
  .row:focus-visible {
    outline: 0;
    background: var(--accent-soft);
    box-shadow: inset 3px 0 0 var(--accent);
  }
  .row.disk {
    min-height: 56px;
    background: color-mix(in srgb, var(--surface-2) 70%, transparent);
  }
  .row.volume {
    min-height: 48px;
  }
  .row.quiet:hover {
    background: transparent;
  }
  .name {
    display: flex;
    align-items: center;
    gap: var(--s-2);
    min-width: 0;
  }
  .indent {
    flex: none;
  }
  .chev {
    display: grid;
    place-items: center;
    flex: none;
    width: 16px;
    color: var(--text-3);
    transition: transform 0.12s ease;
  }
  .chev.turn {
    color: var(--accent);
    transform: rotate(90deg);
  }
  .chev.none {
    visibility: hidden;
  }
  .text {
    display: flex;
    flex-direction: column;
    min-width: 0;
  }
  .text small {
    font-family: var(--font-ui);
    font-size: var(--fs-label);
  }
  .disk b,
  .volume b {
    font-weight: 650;
  }
  .row :global(svg) {
    flex: none;
    color: var(--text-2);
  }
  .disk > .name > :global(svg) {
    color: var(--accent);
  }
  .bar {
    min-width: 0;
  }
  .muted {
    color: var(--text-2);
  }
  .mini {
    padding: 2px 10px;
    font-size: 11px;
  }
  .note {
    padding-top: 8px;
    padding-bottom: 8px;
    border-bottom: 1px solid var(--border);
    color: var(--text-3);
    font-size: var(--fs-small);
  }
  .note.warn {
    color: var(--warn);
  }
  .note.danger {
    color: var(--danger);
  }
  .spinner {
    flex: none;
    width: 12px;
    height: 12px;
    border: 2px solid var(--border-strong);
    border-top-color: var(--accent);
    border-radius: 50%;
    animation: turn 0.7s linear infinite;
  }
  @keyframes turn {
    to {
      transform: rotate(360deg);
    }
  }
  @media (max-width: 900px) {
    .head,
    .row {
      grid-template-columns: minmax(200px, 1fr) 0 90px 52px 0;
    }
    .head span:nth-child(2),
    .head span:nth-child(5),
    .bar,
    .row > span:last-child {
      display: none;
    }
  }
</style>
