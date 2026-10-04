<script lang="ts">
  import { onDestroy, onMount } from "svelte";

  import type { AudioDevice } from "$lib/api/types";
  import Badge from "$lib/components/ui/Badge.svelte";
  import Meter from "$lib/components/ui/Meter.svelte";
  import StatTiles from "$lib/components/ui/StatTiles.svelte";
  import { audioMonitor } from "$lib/features/monitor/audio.svelte";

  const a = $derived(audioMonitor.latest);
  const defaultSink = $derived(a?.sinks.find((s) => s.default) ?? a?.sinks[0]);
  const defaultSource = $derived(a?.sources.find((s) => s.default) ?? a?.sources[0]);

  const short = (name: string) => (name.length > 28 ? `${name.slice(0, 27)}…` : name);
  const kHz = (rate: number | null) =>
    rate === null ? "" : `${(rate / 1000).toFixed(1).replace(/\.0$/, "")} kHz`;

  onMount(audioMonitor.start);
  onDestroy(audioMonitor.stop);
</script>

{#if audioMonitor.error && !a}
  <p class="error">Couldn't read the audio system: {audioMonitor.error}</p>
{:else if !a}
  <div class="skeleton" style="height: 96px; margin-bottom: 16px"></div>
  <div class="skeleton" style="height: 320px"></div>
{:else}
  <StatTiles
    items={[
      {
        label: "Output",
        value: defaultSink ? `${defaultSink.volume}%` : "—",
        sub: defaultSink ? (defaultSink.muted ? "muted" : short(defaultSink.name)) : "none found",
        tone: defaultSink?.muted ? "warn" : undefined,
      },
      {
        label: "Input",
        value: defaultSource ? `${defaultSource.volume}%` : "—",
        sub: defaultSource
          ? defaultSource.muted
            ? "muted"
            : short(defaultSource.name)
          : "none found",
        tone: defaultSource?.muted ? "warn" : undefined,
      },
      {
        label: "Playing",
        value: String(a.playing),
        sub: a.playing === 1 ? "stream" : "streams",
        tone: a.playing ? "ok" : undefined,
      },
      {
        label: "Recording",
        value: String(a.capturing),
        sub: a.capturing ? "something is listening" : "nothing is listening",
        tone: a.capturing ? "warn" : undefined,
      },
    ]}
  />

  <section class="card block">
    <h2>Live streams</h2>
    {#if a.streams.length}
      <ul class="streams">
        {#each a.streams as s (s.card + s.name + s.direction)}
          <li>
            <Badge tone={s.direction === "playback" ? "ok" : "warn"}>
              {s.direction === "playback" ? "Playing" : "Recording"}
            </Badge>
            <span class="who">
              <b>{s.program ?? "Unknown program"}</b>
              <small>{s.name} · {s.card}</small>
            </span>
            <span class="tabular spec">
              {[kHz(s.rate), s.channels ? `${s.channels} ch` : null, s.format]
                .filter(Boolean)
                .join(" · ")}
            </span>
          </li>
        {/each}
      </ul>
    {:else}
      <p class="muted">Nothing is playing or recording right now.</p>
    {/if}
  </section>

  <div class="pair">
    {#snippet devices(title: string, list: AudioDevice[])}
      <section class="card block">
        <h2>{title}</h2>
        {#if list.length}
          <ul class="devs">
            {#each list as d (d.name)}
              <li>
                <span class="who">
                  <b class="truncate" title={d.name}>{d.name}</b>
                  <span class="flags">
                    {#if d.default}<Badge tone="ok">Default</Badge>{/if}
                    {#if d.muted}<Badge tone="warn">Muted</Badge>{/if}
                  </span>
                </span>
                <span class="vol">
                  <span class="tabular">{d.volume}%</span>
                  <Meter value={Math.min(d.volume, 100)} tone="accent" height={5} />
                </span>
              </li>
            {/each}
          </ul>
        {:else}
          <p class="muted">None found.</p>
        {/if}
      </section>
    {/snippet}
    {@render devices("Outputs", a.sinks)}
    {@render devices("Inputs", a.sources)}
  </div>

  <section class="card block">
    <h2>Sound cards</h2>
    <ul class="cards">
      {#each a.cards as c (c.index)}
        <li>
          <span class="tabular muted">card{c.index}</span>
          <b>{c.name}</b>
          <span class="muted">{c.driver}</span>
        </li>
      {/each}
    </ul>
  </section>
{/if}

<style>
  .block {
    margin-bottom: var(--s-4);
    padding: var(--s-4) var(--s-5);
  }
  .block h2 {
    margin-bottom: var(--s-3);
    color: var(--accent);
    font-family: var(--font-mono);
    font-size: var(--fs-label);
    font-weight: 700;
    letter-spacing: 0.06em;
    text-transform: uppercase;
  }
  .pair {
    display: grid;
    grid-template-columns: repeat(2, minmax(0, 1fr));
    gap: var(--s-4);
    margin-bottom: var(--s-4);
  }
  .pair .block {
    margin: 0;
  }
  @media (max-width: 900px) {
    .pair {
      grid-template-columns: minmax(0, 1fr);
    }
  }
  ul {
    margin: 0;
    padding: 0;
    list-style: none;
  }
  .streams li,
  .devs li,
  .cards li {
    display: flex;
    align-items: center;
    gap: var(--s-4);
    padding: 8px 0;
    border-bottom: 1px solid var(--border);
  }
  li:last-child {
    border-bottom: 0 !important;
  }
  .who {
    display: flex;
    flex-direction: column;
    gap: 2px;
    min-width: 0;
    flex: 1;
  }
  .who b {
    font-weight: 650;
  }
  .who small {
    color: var(--text-2);
    font-size: var(--fs-label);
  }
  .flags {
    display: flex;
    gap: var(--s-2);
  }
  .spec {
    color: var(--text-2);
    font-size: var(--fs-small);
    white-space: nowrap;
  }
  .vol {
    display: flex;
    align-items: center;
    gap: var(--s-3);
    width: 180px;
  }
  .vol :global(.meter) {
    flex: 1;
  }
  .cards li b {
    font-weight: 650;
  }
  .muted {
    color: var(--text-2);
  }
  .error {
    padding: var(--s-6) 0;
    color: var(--danger);
    text-align: center;
  }
</style>
