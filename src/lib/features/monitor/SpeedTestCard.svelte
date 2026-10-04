<script lang="ts">
  import { api, errorMessage } from "$lib/api/client";
  import type { SpeedTest } from "$lib/api/types";

  let result = $state.raw<SpeedTest | null>(null);
  let testedAt = $state<string | null>(null);
  let running = $state(false);
  let error = $state<string | null>(null);

  async function run() {
    running = true;
    error = null;
    try {
      result = await api.speedTest();
      testedAt = new Date().toLocaleTimeString();
    } catch (e) {
      error = errorMessage(e);
    } finally {
      running = false;
    }
  }

  const mbit = (bps: number | null) =>
    bps === null ? "—" : `${((bps * 8) / 1e6).toFixed(bps * 8 >= 1e8 ? 0 : 1)} Mbit/s`;
</script>

<section class="card test">
  <header>
    <div>
      <h2>Internet speed test</h2>
      <p>
        The charts above show what is being used; this measures what the line can do, by really
        downloading and uploading for ten seconds each — a few hundred megabytes on a fast
        connection.
      </p>
    </div>
    <button class="btn" onclick={run} disabled={running}>
      {running ? "Measuring… about half a minute" : result ? "Test again" : "Run test"}
    </button>
  </header>

  {#if error}
    <p class="error">The test failed: {error}</p>
  {:else if result}
    <div class="results">
      <div>
        <p class="label">Download</p>
        <p class="value tabular">{mbit(result.download_bps)}</p>
      </div>
      <div>
        <p class="label">Upload</p>
        <p class="value tabular">{mbit(result.upload_bps)}</p>
      </div>
      <div>
        <p class="label">Latency</p>
        <p class="value tabular">
          {result.latency_ms === null ? "—" : `${result.latency_ms.toFixed(0)} ms`}
        </p>
      </div>
      <div>
        <p class="label">Against</p>
        <p class="value small">{result.server}<small>at {testedAt}</small></p>
      </div>
    </div>
  {/if}
</section>

<style>
  .test {
    margin: 0;
    padding: var(--s-4) var(--s-5);
  }
  header {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    justify-content: space-between;
    gap: var(--s-3);
  }
  h2 {
    margin-bottom: var(--s-1);
    color: var(--accent);
    font-family: var(--font-mono);
    font-size: var(--fs-label);
    font-weight: 700;
    letter-spacing: 0.06em;
    text-transform: uppercase;
  }
  header p {
    max-width: 72ch;
    color: var(--text-3);
    font-size: var(--fs-small);
  }
  .results {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(140px, 1fr));
    gap: var(--s-4);
    margin-top: var(--s-4);
  }
  .label {
    color: var(--text-3);
    font-size: var(--fs-label);
    font-weight: 600;
    letter-spacing: 0.06em;
    text-transform: uppercase;
  }
  .value {
    font-size: 22px;
    font-weight: 650;
  }
  .value.small {
    font-size: var(--fs-body);
  }
  .value small {
    display: block;
    color: var(--text-2);
    font-size: var(--fs-label);
    font-weight: 400;
  }
  .error {
    margin-top: var(--s-3);
    color: var(--danger);
  }
</style>
