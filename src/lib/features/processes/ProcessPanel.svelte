<script lang="ts">
  import { fly } from "svelte/transition";
  import Badge from "$lib/components/ui/Badge.svelte";
  import Meter from "$lib/components/ui/Meter.svelte";
  import type { ProcessDetail, ProcessRow } from "$lib/api/types";
  import DetailList from "$lib/components/ui/DetailList.svelte";
  import { groupBySection } from "$lib/utils/details";
  import { formatBytes } from "$lib/utils/format";

  let {
    row,
    detail,
    onclose,
    onselect,
  }: {
    row: ProcessRow | undefined;
    detail: ProcessDetail | null;
    onclose: () => void;
    onselect: (pid: number) => void;
  } = $props();

  const m = $derived(detail?.memory ?? null);
  const used = $derived(m && m.requested > 0 ? (m.resident / m.requested) * 100 : 0);

  // What the resident memory is made of.
  const parts = $derived(
    m
      ? [
          {
            label: "Private data",
            hint: "heap, stack: only this process uses it",
            value: m.anonymous,
            color: "var(--accent)",
          },
          {
            label: "Files and libraries",
            hint: "program code and mapped files",
            value: m.file_backed,
            color: "var(--ok)",
          },
          {
            label: "Shared with others",
            hint: "shared memory",
            value: m.shared,
            color: "var(--warn)",
          },
        ]
      : [],
  );
  const partsTotal = $derived(parts.reduce((n, p) => n + p.value, 0) || 1);

  // The remaining rows, minus what the visuals above already show.
  const sections = $derived(
    groupBySection((detail?.details ?? []).filter((r) => r.section !== "Memory requests")),
  );

  const keyNumbers = $derived(
    m
      ? [
          { label: "Peak in RAM", value: formatBytes(m.peak_resident) },
          { label: "Peak asked for", value: formatBytes(m.peak_requested) },
          ...(m.proportional !== null
            ? [
                {
                  label: "Fair share (PSS)",
                  value: formatBytes(m.proportional),
                  hint: "Private memory plus a fair share of shared pages",
                },
              ]
            : []),
          ...(m.private !== null ? [{ label: "Private", value: formatBytes(m.private) }] : []),
          ...(m.shared_pages !== null
            ? [{ label: "Shared pages", value: formatBytes(m.shared_pages) }]
            : []),
          { label: "Moved to swap", value: formatBytes(m.swapped) },
        ]
      : [],
  );

  const topRegion = $derived(Math.max(1, ...(detail?.regions ?? []).map((r) => r.resident)));
</script>

<aside class="card panel" transition:fly={{ x: 24, duration: 180 }} aria-label="Process details">
  <header>
    <div class="title">
      <h2 class="truncate" title={row?.name}>{row?.name ?? `Process ${detail?.pid ?? ""}`}</h2>
      <p>
        PID {detail?.pid ?? row?.pid}{row ? ` · ${row.user}` : ""}
      </p>
      {#if row}
        <div class="badges">
          <Badge tone={row.state === "Running" ? "ok" : "neutral"}>{row.state}</Badge>
          {#if row.kernel}<Badge>Kernel</Badge>{/if}
        </div>
      {/if}
    </div>
    <button class="close" onclick={onclose} aria-label="Close">
      <svg
        width="16"
        height="16"
        viewBox="0 0 24 24"
        fill="none"
        stroke="currentColor"
        stroke-width="2"
        stroke-linecap="round"><path d="M18 6 6 18M6 6l12 12" /></svg
      >
    </button>
  </header>

  <div class="body">
    {#if !detail}
      <p class="muted pad">Reading…</p>
    {:else if !detail.running}
      <p class="muted pad">This process has ended.</p>
    {:else}
      {#if m}
        <section>
          <h3>Memory requests</h3>
          <div class="ask">
            <div class="line">
              <span>Asked for from the system</span><strong class="tabular"
                >{formatBytes(m.requested)}</strong
              >
            </div>
            <Meter value={100} tone="accent" height={8} />
            <div class="line">
              <span>Actually in RAM</span><strong class="tabular">{formatBytes(m.resident)}</strong>
            </div>
            <Meter value={used} tone="ok" height={8} />
            <p class="note">
              Programs reserve far more address space than they touch. This one uses
              <strong>{used.toFixed(1)}%</strong> of what it asked for.
            </p>
          </div>
        </section>

        <section>
          <h3>What's in RAM</h3>
          <div class="composition" role="img" aria-label="Memory composition">
            {#each parts as p}
              <span
                style:width="{(p.value / partsTotal) * 100}%"
                style:background={p.color}
                title="{p.label}: {formatBytes(p.value)}"
              ></span>
            {/each}
          </div>
          <ul class="legend">
            {#each parts as p}
              <li>
                <i style:background={p.color}></i>
                <span title={p.hint}>{p.label}</span>
                <b class="tabular">{formatBytes(p.value)}</b>
              </li>
            {/each}
          </ul>
        </section>

        <section>
          <h3>Key numbers</h3>
          <DetailList rows={keyNumbers} />
        </section>
      {/if}

      {#if detail.regions.length}
        <section>
          <h3>Largest memory regions</h3>
          <ul class="regions">
            {#each detail.regions as r}
              <li>
                <div class="line">
                  <span class="truncate" title={r.name}>{r.name.split("/").pop() || r.name}</span>
                  <b class="tabular">{formatBytes(r.resident)}</b>
                </div>
                <Meter value={(r.resident / topRegion) * 100} tone="accent" height={4} />
                <small class="tabular">{formatBytes(r.size)} reserved</small>
              </li>
            {/each}
          </ul>
        </section>
      {/if}

      {#if detail.children.length}
        <section>
          <h3>Started by this process ({detail.children.length})</h3>
          <ul class="children">
            {#each detail.children.slice(0, 12) as c (c.pid)}
              <li>
                <button onclick={() => onselect(c.pid)}>
                  <span class="truncate">{c.name}</span>
                  <small class="tabular">{c.pid} · {formatBytes(c.memory)}</small>
                </button>
              </li>
            {/each}
          </ul>
        </section>
      {/if}

      {#each sections as s}
        <section>
          <h3>{s.title}</h3>
          <DetailList rows={s.rows} stackAt={48} />
        </section>
      {/each}
    {/if}
  </div>
</aside>

<style>
  .panel {
    display: flex;
    flex: none;
    flex-direction: column;
    align-self: stretch;
    width: 400px;
    margin: 0;
    border-width: 0 0 0 1px;
    overflow: hidden;
  }
  header {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    gap: var(--s-3);
    padding: var(--s-4);
    border-bottom: 1px solid var(--border);
  }
  .title {
    min-width: 0;
  }
  h2 {
    font-size: 17px;
    font-weight: 650;
  }
  .title p {
    color: var(--text-2);
  }
  .badges {
    display: flex;
    gap: var(--s-2);
    margin-top: var(--s-2);
  }
  .close {
    display: grid;
    place-items: center;
    flex: none;
    width: 28px;
    height: 28px;
    border: 0;
    border-radius: 0;
    background: var(--surface-2);
    color: var(--text-2);
    cursor: pointer;
  }
  .close:hover {
    color: var(--text);
  }

  .body {
    flex: 1;
    overflow-y: auto;
    padding: 0 var(--s-4) var(--s-4);
  }
  section {
    margin-top: var(--s-4);
  }
  h3 {
    margin-bottom: var(--s-2);
    font-size: var(--fs-label);
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.06em;
    color: var(--text-3);
  }
  .muted {
    color: var(--text-3);
  }
  .pad {
    padding: var(--s-5) 0;
    text-align: center;
  }

  .ask {
    display: flex;
    flex-direction: column;
    gap: var(--s-2);
  }
  .line {
    display: flex;
    justify-content: space-between;
    gap: var(--s-3);
    min-width: 0;
  }
  .line span {
    color: var(--text-2);
  }
  .note {
    margin-top: var(--s-1);
    color: var(--text-2);
    font-size: var(--fs-small);
  }

  .composition {
    display: flex;
    height: 12px;
    border-radius: 0;
    background: var(--surface-2);
    overflow: hidden;
  }
  .composition span {
    min-width: 2px;
    transition: width 0.4s ease;
  }
  .legend {
    margin: var(--s-3) 0 0;
    padding: 0;
    list-style: none;
  }
  .legend li {
    display: flex;
    align-items: center;
    gap: var(--s-2);
    padding: 3px 0;
  }
  .legend i {
    width: 10px;
    height: 10px;
    border-radius: 0;
  }
  .legend span {
    flex: 1;
    color: var(--text-2);
  }

  .regions,
  .children {
    margin: 0;
    padding: 0;
    list-style: none;
  }
  .regions li {
    padding: 6px 0;
  }
  .regions small {
    color: var(--text-3);
    font-size: var(--fs-small);
  }
  .children button {
    display: flex;
    justify-content: space-between;
    gap: var(--s-3);
    width: 100%;
    padding: 6px var(--s-2);
    border: 0;
    border-radius: 0;
    background: transparent;
    text-align: left;
    cursor: pointer;
  }
  .children button:hover {
    background: var(--surface-2);
  }
  .children small {
    flex: none;
    color: var(--text-3);
  }
  @media (max-width: 1050px) {
    .panel {
      position: absolute;
      right: 0;
      top: 0;
      bottom: 0;
      z-index: 10;
      width: min(400px, 100%);
      box-shadow: -12px 0 32px rgb(0 0 0 / 0.2);
    }
  }
</style>
