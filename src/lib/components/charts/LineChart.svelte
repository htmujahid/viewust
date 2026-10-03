<script lang="ts">
  import { niceMax } from "./scale";

  /**
   * A live line chart: one shared y-axis, newest sample on the right.
   * Specs: 2px lines, a faint wash under the first series, an 8px end dot with a
   * surface ring, hairline grid, a hover/keyboard crosshair that reads out every
   * series, a legend whenever there are two or more series, and a table view.
   */
  type Series = { name: string; color: string; values: (number | null)[] };

  let {
    series,
    format,
    max,
    height = 130,
    window = 60,
    label,
    table = true,
    floor = 0,
  }: {
    series: Series[];
    format: (v: number) => string;
    /** Fix the top of the axis (e.g. 100 for percentages); otherwise it follows the data. */
    max?: number;
    height?: number;
    /** Samples shown across the width. */
    window?: number;
    /** Accessible name for the chart. */
    label: string;
    /** Smallest value the axis may top out at, so a flat line isn't blown up to fill it. */
    floor?: number;
    /** Offer the "Show as table" view. */
    table?: boolean;
  } = $props();

  let width = $state(0);
  let hover = $state<number | null>(null);
  let showTable = $state(false);

  const top = $derived(
    max ??
      niceMax(
        Math.max(floor, ...series.flatMap((s) => s.values.filter((v): v is number => v !== null))),
      ),
  );
  // room for the widest axis label ("100 MB/s" is wider than "50%")
  const leftPad = $derived(Math.max(46, format(top).length * 6.8 + 16));
  const pad = $derived({ left: leftPad, right: 12, top: 10, bottom: 22 });
  const plotW = $derived(Math.max(width - pad.left - pad.right, 10));
  const plotH = $derived(height - pad.top - pad.bottom);
  const count = $derived(Math.max(...series.map((s) => s.values.length), 0));
  // newest sample sits at the right edge; the line grows in from the right
  const step = $derived(plotW / Math.max(window - 1, 1));
  const xOf = (i: number) => pad.left + plotW - (count - 1 - i) * step;

  const yOf = (v: number) => pad.top + plotH - (Math.min(v, top) / top) * plotH;
  const ticks = $derived([0, 0.5, 1].map((f) => ({ v: top * f, y: yOf(top * f) })));

  function path(values: (number | null)[]): string {
    let d = "";
    let pen = false;
    values.forEach((v, i) => {
      if (v === null) {
        pen = false;
        return;
      }
      d += `${pen ? "L" : "M"}${xOf(i).toFixed(1)} ${yOf(v).toFixed(1)}`;
      pen = true;
    });
    return d;
  }
  const area = (values: (number | null)[]) => {
    const pts = values
      .map((v, i) => (v === null ? null : [xOf(i), yOf(v)]))
      .filter(Boolean) as number[][];
    if (pts.length < 2) return "";
    const base = pad.top + plotH;
    return `M${pts[0][0]} ${base}L${pts.map((p) => `${p[0].toFixed(1)} ${p[1].toFixed(1)}`).join("L")}L${pts[pts.length - 1][0]} ${base}Z`;
  };

  function indexFromPointer(e: PointerEvent) {
    const box = (e.currentTarget as SVGElement).getBoundingClientRect();
    const x = e.clientX - box.left;
    const i = Math.round(count - 1 - (pad.left + plotW - x) / step);
    hover = Math.min(Math.max(i, 0), count - 1);
  }
  function onkeydown(e: KeyboardEvent) {
    if (e.key === "ArrowLeft") hover = Math.max((hover ?? count - 1) - 1, 0);
    else if (e.key === "ArrowRight") hover = Math.min((hover ?? count - 1) + 1, count - 1);
    else if (e.key === "Escape") hover = null;
    else return;
    e.preventDefault();
  }

  const shown = $derived(hover ?? count - 1);
  const secondsAgo = (i: number) => count - 1 - i;
  const tipLeft = $derived(
    hover === null ? 0 : Math.min(Math.max(xOf(hover), 70), Math.max(width - 70, 70)),
  );
  const rows = $derived(Array.from({ length: Math.min(count, 12) }, (_, k) => count - 1 - k));
</script>

<div class="chart">
  {#if series.length >= 2}
    <ul class="legend">
      {#each series as s}
        <li><i style:background={s.color}></i>{s.name}</li>
      {/each}
    </ul>
  {/if}

  <div class="plot" bind:clientWidth={width} style:height="{height}px">
    {#if width > 0 && count > 0}
      <!-- svelte-ignore a11y_no_noninteractive_tabindex, a11y_no_noninteractive_element_interactions -->
      <svg
        {width}
        {height}
        role="img"
        aria-label={label}
        tabindex="0"
        onpointermove={indexFromPointer}
        onpointerleave={() => (hover = null)}
        {onkeydown}
        onblur={() => (hover = null)}
      >
        {#each ticks as t}
          <line class="grid" x1={pad.left} x2={pad.left + plotW} y1={t.y} y2={t.y} />
          <text class="tick" x={pad.left - 8} y={t.y + 4} text-anchor="end">{format(t.v)}</text>
        {/each}
        <text class="tick" x={pad.left} y={height - 4}>{window}s ago</text>
        <text class="tick" x={pad.left + plotW} y={height - 4} text-anchor="end">now</text>

        {#if series[0]}
          <path d={area(series[0].values)} fill={series[0].color} opacity="0.14" />
        {/if}
        {#each series as s}
          <path class="line" d={path(s.values)} stroke={s.color} style:color={s.color} />
        {/each}

        {#if hover !== null}
          <line class="cross" x1={xOf(hover)} x2={xOf(hover)} y1={pad.top} y2={pad.top + plotH} />
        {/if}
        {#each series as s}
          {@const v = s.values[shown]}
          {#if v !== null && v !== undefined}
            <circle class="dot" cx={xOf(shown)} cy={yOf(v)} r="4" fill={s.color} />
          {/if}
        {/each}
      </svg>

      {#if hover !== null}
        <div class="tip" style:left="{tipLeft}px" role="status">
          <p class="when">{secondsAgo(hover) === 0 ? "Now" : `${secondsAgo(hover)}s ago`}</p>
          {#each series as s}
            {@const v = s.values[hover]}
            <p class="row">
              <i style:background={s.color}></i>
              <strong>{v === null || v === undefined ? "—" : format(v)}</strong>
              <span>{s.name}</span>
            </p>
          {/each}
        </div>
      {/if}
    {/if}
  </div>

  {#if table}
    <button class="toggle" onclick={() => (showTable = !showTable)} aria-expanded={showTable}>
      {showTable ? "Hide table" : "Show as table"}
    </button>
  {/if}
  {#if table && showTable}
    <table>
      <thead>
        <tr>
          <th>When</th>
          {#each series as s}<th>{s.name}</th>{/each}
        </tr>
      </thead>
      <tbody>
        {#each rows as i}
          <tr>
            <td>{secondsAgo(i) === 0 ? "Now" : `${secondsAgo(i)}s ago`}</td>
            {#each series as s}
              <td class="tabular"
                >{s.values[i] === null || s.values[i] === undefined
                  ? "—"
                  : format(s.values[i] as number)}</td
              >
            {/each}
          </tr>
        {/each}
      </tbody>
    </table>
  {/if}
</div>

<style>
  .chart {
    min-width: 0;
  }
  .legend {
    display: flex;
    flex-wrap: wrap;
    gap: var(--s-4);
    margin: 0 0 var(--s-2);
    padding: 0;
    list-style: none;
    color: var(--text-2);
    font-size: var(--fs-small);
  }
  .legend li {
    display: inline-flex;
    align-items: center;
    gap: 6px;
  }
  .legend i,
  .row i {
    display: inline-block;
    width: 14px;
    height: 2px;
    border-radius: 0;
  }
  .plot {
    position: relative;
  }
  svg {
    display: block;
    outline: none;
    touch-action: none;
  }
  svg:focus-visible {
    outline: 2px solid var(--accent);
    outline-offset: 2px;
    border-radius: 0;
  }
  .grid {
    stroke: var(--border);
    stroke-width: 1;
    stroke-dasharray: 2 4;
  }
  .cross {
    stroke: var(--text-3);
    stroke-width: 1;
  }
  .tick {
    fill: var(--text-3);
    font-family: var(--font-mono);
    font-size: 11px;
    font-variant-numeric: tabular-nums;
  }
  .line {
    fill: none;
    filter: drop-shadow(0 0 3px currentColor);
    stroke-width: 2;
    stroke-linejoin: round;
    stroke-linecap: round;
  }
  .dot {
    stroke: var(--surface);
    stroke-width: 2;
  }
  .tip {
    position: absolute;
    top: 0;
    z-index: 2;
    min-width: 150px;
    padding: var(--s-2) var(--s-3);
    border: 1px solid var(--accent);
    border-radius: 0;
    background: var(--surface);
    box-shadow: var(--glow);
    pointer-events: none;
    transform: translate(-50%, -4px);
  }
  .tip p {
    margin: 0;
  }
  .when {
    color: var(--text-3);
    font-size: var(--fs-label);
  }
  .row {
    display: flex;
    align-items: center;
    gap: 6px;
    font-size: var(--fs-small);
  }
  .row strong {
    font-variant-numeric: tabular-nums;
    white-space: nowrap;
  }
  .row span {
    color: var(--text-2);
  }
  .toggle {
    margin-top: var(--s-1);
    padding: 2px 0;
    border: 0;
    background: transparent;
    color: var(--text-3);
    font-size: var(--fs-small);
    cursor: pointer;
  }
  .toggle:hover {
    color: var(--text-2);
  }
  table {
    width: 100%;
    margin-top: var(--s-2);
    border-collapse: collapse;
    font-size: var(--fs-small);
  }
  th,
  td {
    padding: 4px var(--s-2);
    border-bottom: 1px solid var(--border);
    text-align: left;
  }
  th {
    color: var(--text-3);
    font-weight: 600;
  }
  td:not(:first-child),
  th:not(:first-child) {
    text-align: right;
  }
</style>
