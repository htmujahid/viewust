<script lang="ts">
  import { onDestroy, onMount } from "svelte";

  import { goto } from "$app/navigation";

  import Badge from "$lib/components/ui/Badge.svelte";
  import DataTable, { type Column } from "$lib/components/ui/DataTable.svelte";
  import DetailList from "$lib/components/ui/DetailList.svelte";
  import FilterChips from "$lib/components/ui/FilterChips.svelte";
  import SearchBox from "$lib/components/ui/SearchBox.svelte";
  import SectionShell from "$lib/components/ui/SectionShell.svelte";
  import SidePanel from "$lib/components/ui/SidePanel.svelte";
  import StatTiles from "$lib/components/ui/StatTiles.svelte";
  import { groupMenu } from "$lib/features/accounts/menu";
  import { menu } from "$lib/stores/menu.svelte";

  import type { GroupFilter } from "$lib/features/accounts/logic";
  import { accounts } from "$lib/features/accounts/store.svelte";
  import type { GroupRow } from "$lib/api/types";

  const snapshot = $derived(accounts.snapshot);
  const rows = $derived(accounts.groups);
  const selected = $derived(snapshot?.groups.find((g) => g.name === accounts.selectedGroup));

  const filters = $derived<{ key: GroupFilter; label: string; count: number }[]>([
    { key: "all", label: "All", count: accounts.countGroups("all") },
    { key: "admin", label: "Admin", count: accounts.countGroups("admin") },
    { key: "regular", label: "Personal", count: accounts.countGroups("regular") },
    { key: "system", label: "System", count: accounts.countGroups("system") },
    { key: "empty", label: "Empty", count: accounts.countGroups("empty") },
  ]);

  const columns: Column[] = [
    { key: "name", label: "Group" },
    { key: "gid", label: "ID", right: true },
    { key: "kind", label: "Kind" },
    { key: "members", label: "Members" },
  ];

  const kindBadge = (g: GroupRow) =>
    g.admin
      ? { label: "Admin", tone: "warn" as const }
      : g.kind === "regular"
        ? { label: "Personal", tone: "neutral" as const }
        : { label: "System", tone: "neutral" as const };

  const membersText = (g: GroupRow) =>
    g.members.length === 0
      ? "—"
      : g.members.length <= 3
        ? g.members.join(", ")
        : `${g.members.slice(0, 3).join(", ")} +${g.members.length - 3}`;

  function onkeydown(e: KeyboardEvent) {
    if (e.key !== "Escape") return;
    if (accounts.selectedGroup !== null) accounts.selectedGroup = null;
    else goto("/os");
  }

  onMount(accounts.start);
  onDestroy(accounts.stop);
</script>

<svelte:window {onkeydown} />

<SectionShell title="Groups">
  {#snippet actions()}
    <SearchBox
      bind:value={accounts.groupSearch}
      placeholder="Search group, ID or member"
      label="Search groups"
    />
  {/snippet}

  {#if accounts.error && !snapshot}
    <p class="error">Couldn't read the groups: {accounts.error}</p>
  {:else if !snapshot}
    <div class="skeleton" style="height: 96px; margin-bottom: 16px"></div>
    <div class="skeleton" style="height: 420px"></div>
  {:else}
    <StatTiles
      items={[
        { label: "Groups", value: String(snapshot.groups.length) },
        {
          label: "Admin",
          value: String(accounts.countGroups("admin")),
          sub: "full control",
          tone: "warn",
        },
        {
          label: "In use",
          value: String(snapshot.groups.length - accounts.countGroups("empty")),
          sub: "have members",
        },
        { label: "Empty", value: String(accounts.countGroups("empty")) },
      ]}
    />

    <div class="toolbar">
      <FilterChips options={filters} bind:value={accounts.groupFilter} label="Filter groups" />
      <p class="muted">{rows.length} {rows.length === 1 ? "group" : "groups"}</p>
    </div>

    <DataTable
      {rows}
      {columns}
      rowKey={(g) => g.name}
      selectedKey={accounts.selectedGroup}
      sortKey={accounts.groupSort}
      sortDesc={accounts.groupDesc}
      emptyText={accounts.groupSearch
        ? `No groups match “${accounts.groupSearch}”.`
        : "No groups in this filter."}
      onsort={(k) => accounts.sortGroups(k)}
      onselect={(k) => (accounts.selectedGroup = k)}
      oncontext={(g, e) => menu.open(e, groupMenu(g), g.name)}
    >
      {#snippet cell(g, c)}
        {#if c.key === "name"}
          <b>{g.name}</b>
        {:else if c.key === "gid"}
          <span class="muted">{g.gid}</span>
        {:else if c.key === "kind"}
          <Badge tone={kindBadge(g).tone}>{kindBadge(g).label}</Badge>
        {:else}
          <span class="muted">{membersText(g)}</span>
        {/if}
      {/snippet}
    </DataTable>
  {/if}

  {#snippet panel()}
    {#if selected}
      <SidePanel
        title={selected.name}
        subtitle="Group ID {selected.gid}"
        label="Group details"
        onclose={() => (accounts.selectedGroup = null)}
      >
        {#snippet badges()}
          <Badge tone={kindBadge(selected).tone}>{kindBadge(selected).label}</Badge>
        {/snippet}
        <section>
          <h3>Group</h3>
          <DetailList
            rows={[
              { label: "Group ID", value: selected.gid },
              { label: "Members", value: String(selected.members.length) },
            ]}
          />
        </section>
        <section>
          <h3>Members</h3>
          <div class="members">
            {#each selected.members as m}
              <button
                class="member"
                onclick={() => {
                  accounts.openUser(m);
                  goto("/accounts");
                }}>{m}</button
              >
            {:else}
              <p class="muted">Nobody belongs to this group.</p>
            {/each}
          </div>
        </section>
      </SidePanel>
    {/if}
  {/snippet}
</SectionShell>

<style>
  .toolbar {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    justify-content: space-between;
    gap: var(--s-3);
    margin-bottom: var(--s-2);
  }
  .muted {
    color: var(--text-2);
  }
  .members {
    display: flex;
    flex-wrap: wrap;
    gap: var(--s-2);
  }
  .member {
    padding: 3px var(--s-3);
    border: 1px solid var(--border);
    background: var(--surface-2);
    color: var(--text-2);
    font-family: var(--font-mono);
    font-size: 11px;
    cursor: pointer;
  }
  .member:hover {
    border-color: var(--accent);
    color: var(--accent);
    box-shadow: var(--glow);
  }
  .error {
    padding: var(--s-6) 0;
    color: var(--danger);
    text-align: center;
  }
</style>
