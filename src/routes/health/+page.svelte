<script lang="ts">
  import { onMount } from "svelte";

  import { goto } from "$app/navigation";

  import { api, errorMessage } from "$lib/api/client";
  import type { HealthReport, Severity } from "$lib/api/types";
  import Badge from "$lib/components/ui/Badge.svelte";
  import Page from "$lib/components/ui/Page.svelte";
  import RescanButton from "$lib/components/ui/RescanButton.svelte";

  let report = $state.raw<HealthReport | null>(null);
  let error = $state<string | null>(null);
  let loading = $state(false);

  async function load() {
    loading = true;
    try {
      report = await api.healthReport();
      error = null;
    } catch (e) {
      error = errorMessage(e);
    } finally {
      loading = false;
    }
  }

  const TONE: Record<Severity, "danger" | "warn" | "ok"> = {
    danger: "danger",
    warn: "warn",
    ok: "ok",
  };
  const WORD: Record<Severity, string> = { danger: "Problem", warn: "Worth a look", ok: "Fine" };

  const verdict = $derived(
    !report
      ? null
      : report.problems > 0
        ? {
            tone: "danger" as const,
            text:
              report.problems === 1
                ? "1 problem needs attention"
                : `${report.problems} problems need attention`,
          }
        : report.warnings > 0
          ? {
              tone: "warn" as const,
              text: `Nothing broken · ${report.warnings} thing${report.warnings === 1 ? "" : "s"} worth a look`,
            }
          : { tone: "ok" as const, text: "Everything looks healthy" },
  );

  function onkeydown(e: KeyboardEvent) {
    if (e.key === "Escape") goto("/");
  }

  onMount(() => void load());
</script>

<svelte:window {onkeydown} />

<Page>
  <div class="bar">
    <h1>Health</h1>
    <RescanButton {loading} onclick={load} label />
  </div>

  {#if error && !report}
    <p class="error">Couldn't judge the computer: {error}</p>
  {:else if !report}
    <div class="skeleton" style="height: 110px; margin-bottom: 16px"></div>
    <div class="skeleton" style="height: 420px"></div>
  {:else if verdict}
    <section class="card verdict {verdict.tone}">
      <span class="dot" aria-hidden="true"></span>
      <div>
        <h2>{verdict.text}</h2>
        <p>
          {report.checks.length} checks · {report.problems} problems · {report.warnings} warnings ·
          {report.fine} fine
        </p>
      </div>
    </section>

    <div class="checks">
      {#each report.checks as c (c.title + c.detail)}
        <svelte:element this={c.link ? "a" : "div"} class="card check" href={c.link ?? undefined}>
          <Badge tone={TONE[c.severity] === "ok" ? "ok" : TONE[c.severity]}>
            {WORD[c.severity]}
          </Badge>
          <span class="what">
            <b>{c.title}</b>
            <small>{c.detail}</small>
          </span>
          {#if c.link}<span class="go" aria-hidden="true">→</span>{/if}
        </svelte:element>
      {/each}
    </div>
  {/if}
</Page>

<style>
  .bar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: var(--s-3);
    margin-bottom: var(--s-4);
  }
  h1 {
    font-family: var(--font-mono);
    font-size: 20px;
    font-weight: 700;
    text-shadow: var(--glow-text);
  }
  .verdict {
    display: flex;
    align-items: center;
    gap: var(--s-4);
    margin-bottom: var(--s-5);
    padding: var(--s-5);
  }
  .verdict h2 {
    font-family: var(--font-mono);
    font-size: 20px;
    font-weight: 700;
  }
  .verdict p {
    margin-top: var(--s-1);
    color: var(--text-2);
    font-size: var(--fs-small);
  }
  .dot {
    flex: none;
    width: 14px;
    height: 14px;
    border-radius: 50%;
  }
  .verdict.ok .dot {
    background: var(--ok);
    box-shadow: 0 0 14px var(--ok);
  }
  .verdict.ok h2 {
    color: var(--ok);
  }
  .verdict.warn .dot {
    background: var(--warn);
    box-shadow: 0 0 14px var(--warn);
  }
  .verdict.warn h2 {
    color: var(--warn);
  }
  .verdict.danger .dot {
    background: var(--danger);
    box-shadow: 0 0 14px var(--danger);
  }
  .verdict.danger h2 {
    color: var(--danger);
  }
  .checks {
    display: grid;
    grid-template-columns: repeat(2, minmax(0, 1fr));
    gap: var(--s-3);
  }
  @media (max-width: 760px) {
    .checks {
      grid-template-columns: minmax(0, 1fr);
    }
  }
  .check {
    display: flex;
    align-items: center;
    gap: var(--s-3);
    margin: 0;
    padding: var(--s-3) var(--s-4);
    color: var(--text);
    text-decoration: none;
  }
  a.check:hover {
    border-color: var(--accent);
    box-shadow: var(--glow);
  }
  .what {
    display: flex;
    flex-direction: column;
    gap: 2px;
    min-width: 0;
  }
  .what b {
    font-weight: 650;
  }
  .what small {
    color: var(--text-2);
    font-size: var(--fs-small);
  }
  .go {
    margin-left: auto;
    color: var(--text-3);
  }
  .error {
    padding: var(--s-6) 0;
    color: var(--danger);
    text-align: center;
  }
</style>
