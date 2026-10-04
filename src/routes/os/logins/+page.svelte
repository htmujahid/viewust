<script lang="ts">
  import { onMount } from "svelte";

  import { goto } from "$app/navigation";

  import Badge from "$lib/components/ui/Badge.svelte";
  import DataTable, { type Column } from "$lib/components/ui/DataTable.svelte";
  import RescanButton from "$lib/components/ui/RescanButton.svelte";
  import SectionShell from "$lib/components/ui/SectionShell.svelte";
  import StatTiles from "$lib/components/ui/StatTiles.svelte";
  import { osOverview } from "$lib/features/os/overview.svelte";
  import { menu } from "$lib/stores/menu.svelte";
  import { copyText } from "$lib/utils/actions";

  // A stable handle: the Loaded instance itself never changes, only its fields do.
  const logins = osOverview.logins;
  const data = $derived(logins.data);
  const sessions = $derived(data?.sessions ?? []);

  const columns: Column[] = [
    { key: "user", label: "User", sortable: false },
    { key: "kind", label: "Kind", sortable: false },
    { key: "place", label: "From", sortable: false, hint: "The terminal, seat or remote machine" },
    { key: "since", label: "Since", sortable: false },
    { key: "state", label: "State", sortable: false },
  ];

  function onkeydown(e: KeyboardEvent) {
    if (e.key === "Escape") goto("/os");
  }

  onMount(logins.ensure);
</script>

<svelte:window {onkeydown} />

<SectionShell title="Logins">
  {#snippet actions()}
    <RescanButton loading={logins.loading} onclick={logins.load} label />
  {/snippet}

  {#if logins.error && !data}
    <p class="error">Couldn't read the sessions: {logins.error}</p>
  {:else if !data}
    <div class="skeleton" style="height: 96px; margin-bottom: 16px"></div>
    <div class="skeleton" style="height: 320px"></div>
  {:else if !data.available}
    <p class="empty">This computer doesn't use systemd-logind, so there are no sessions to list.</p>
  {:else}
    <StatTiles
      items={[
        { label: "Sessions", value: String(sessions.length), sub: "signed in now" },
        { label: "Users", value: String(new Set(sessions.map((s) => s.user)).size) },
        {
          label: "From elsewhere",
          value: String(sessions.filter((s) => s.remote).length),
          sub: "remote sessions",
          tone: sessions.some((s) => s.remote) ? "warn" : undefined,
        },
      ]}
    />

    <DataTable
      rows={sessions}
      {columns}
      rowKey={(s) => s.id}
      sortKey=""
      sortDesc={false}
      emptyText="Nobody is signed in."
      onsort={() => {}}
      onselect={() => {}}
      oncontext={(s, e) =>
        menu.open(
          e,
          [{ label: "Copy user", onselect: () => copyText(s.user, "user") }],
          `Session ${s.id}`,
        )}
    >
      {#snippet cell(s, c)}
        {#if c.key === "user"}
          <b>{s.user}</b>
          {#if s.class && s.class !== "user"}<Badge>{s.class}</Badge>{/if}
        {:else if c.key === "kind"}
          <span class="muted">{s.kind}</span>
        {:else if c.key === "place"}
          <span class="tabular">{s.place}</span>
          {#if s.remote}<Badge tone="warn">Remote</Badge>{/if}
        {:else if c.key === "since"}
          <span class="tabular muted">{s.since ?? "—"}</span>
        {:else}
          <Badge tone={s.state === "active" ? "ok" : "neutral"}>{s.state}</Badge>
        {/if}
      {/snippet}
    </DataTable>
  {/if}
</SectionShell>

<style>
  .muted {
    color: var(--text-2);
  }
  .empty,
  .error {
    padding: var(--s-6) 0;
    text-align: center;
  }
  .empty {
    color: var(--text-3);
  }
  .error {
    padding: var(--s-6) 0;
    color: var(--danger);
  }
</style>
