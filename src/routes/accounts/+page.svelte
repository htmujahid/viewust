<script lang="ts">
  import { onDestroy, onMount } from "svelte";

  import { goto } from "$app/navigation";

  import Badge from "$lib/components/ui/Badge.svelte";
  import DataTable, { type Column } from "$lib/components/ui/DataTable.svelte";
  import DetailList from "$lib/components/ui/DetailList.svelte";
  import FilterChips from "$lib/components/ui/FilterChips.svelte";
  import SearchBox from "$lib/components/ui/SearchBox.svelte";
  import SidePanel from "$lib/components/ui/SidePanel.svelte";
  import StatTiles from "$lib/components/ui/StatTiles.svelte";
  import { userMenu } from "$lib/features/accounts/menu";
  import { menu } from "$lib/stores/menu.svelte";
  import AccountsShell from "$lib/features/accounts/AccountsShell.svelte";
  import { userBadges, type UserFilter } from "$lib/features/accounts/logic";
  import { accounts } from "$lib/features/accounts/store.svelte";
  import type { UserRow } from "$lib/api/types";
  import { formatBytes } from "$lib/utils/format";

  const snapshot = $derived(accounts.snapshot);
  const rows = $derived(accounts.users);
  const selected = $derived(snapshot?.users.find((u) => u.name === accounts.selectedUser));

  const filters = $derived<{ key: UserFilter; label: string; count: number }[]>([
    { key: "all", label: "All", count: accounts.countUsers("all") },
    { key: "people", label: "People", count: accounts.countUsers("people") },
    { key: "system", label: "System", count: accounts.countUsers("system") },
    { key: "admin", label: "Admins", count: accounts.countUsers("admin") },
    { key: "login", label: "Can log in", count: accounts.countUsers("login") },
  ]);

  const columns: Column[] = [
    { key: "name", label: "User" },
    { key: "uid", label: "ID", right: true },
    { key: "group", label: "Main group" },
    { key: "groups", label: "Groups", right: true },
    { key: "shell", label: "Shell" },
    {
      key: "processes",
      label: "Programs",
      right: true,
      hint: "Programs this account is running now",
    },
    { key: "memory", label: "Memory", right: true, hint: "Memory held by its programs" },
  ];

  const detailRows = (u: UserRow) => [
    { label: "User ID", value: u.uid },
    { label: "Main group", value: u.primary_group },
    ...(u.home ? [{ label: "Home folder", value: u.home }] : []),
    ...(u.shell ? [{ label: "Shell", value: u.shell }] : []),
    ...(u.can_login !== null ? [{ label: "Can log in", value: u.can_login ? "Yes" : "No" }] : []),
    { label: "Running programs", value: String(u.processes) },
    { label: "Memory held", value: formatBytes(u.memory) },
  ];

  function onkeydown(e: KeyboardEvent) {
    if (e.key !== "Escape") return;
    if (accounts.selectedUser !== null) accounts.selectedUser = null;
    else goto("/");
  }

  onMount(accounts.start);
  onDestroy(accounts.stop);
</script>

<svelte:window {onkeydown} />

<AccountsShell>
  {#snippet actions()}
    <SearchBox
      bind:value={accounts.userSearch}
      placeholder="Search name, ID or group"
      label="Search users"
    />
  {/snippet}

  {#if accounts.error && !snapshot}
    <p class="error">Couldn't read the accounts: {accounts.error}</p>
  {:else if !snapshot}
    <div class="skeleton" style="height: 96px; margin-bottom: 16px"></div>
    <div class="skeleton" style="height: 420px"></div>
  {:else}
    <StatTiles
      items={[
        { label: "Accounts", value: String(snapshot.users.length) },
        { label: "People", value: String(accounts.countUsers("people")), sub: "real users" },
        { label: "System", value: String(accounts.countUsers("system")), sub: "for services" },
        { label: "Admins", value: String(accounts.countUsers("admin")), tone: "warn" },
      ]}
    />

    <div class="toolbar">
      <FilterChips options={filters} bind:value={accounts.userFilter} label="Filter users" />
      <p class="muted">{rows.length} {rows.length === 1 ? "account" : "accounts"}</p>
    </div>

    <DataTable
      {rows}
      {columns}
      rowKey={(u) => u.name}
      selectedKey={accounts.selectedUser}
      sortKey={accounts.userSort}
      sortDesc={accounts.userDesc}
      emptyText={accounts.userSearch
        ? `No accounts match “${accounts.userSearch}”.`
        : "No accounts in this group."}
      onsort={(k) => accounts.sortUsers(k)}
      onselect={(k) => (accounts.selectedUser = k)}
      oncontext={(u, e) => menu.open(e, userMenu(u), u.name)}
    >
      {#snippet cell(u, c)}
        {#if c.key === "name"}
          <span class="name">
            <b>{u.name}</b>
            {#each userBadges(u).slice(0, 2) as b}<Badge tone={b.tone}>{b.label}</Badge>{/each}
          </span>
          {#if u.full_name && u.full_name !== u.name}<small class="muted">{u.full_name}</small>{/if}
        {:else if c.key === "uid"}
          <span class="muted">{u.uid}</span>
        {:else if c.key === "group"}
          {u.primary_group}
        {:else if c.key === "groups"}
          <span class="muted">{u.groups.length}</span>
        {:else if c.key === "shell"}
          <span class="muted">{u.shell ?? "—"}</span>
        {:else if c.key === "processes"}
          {u.processes || "—"}
        {:else}
          {u.memory ? formatBytes(u.memory) : "—"}
        {/if}
      {/snippet}
    </DataTable>
  {/if}

  {#snippet panel()}
    {#if selected}
      <SidePanel
        title={selected.name}
        subtitle={selected.full_name && selected.full_name !== selected.name
          ? selected.full_name
          : null}
        label="User details"
        onclose={() => (accounts.selectedUser = null)}
      >
        {#snippet badges()}
          {#each userBadges(selected) as b}<Badge tone={b.tone}>{b.label}</Badge>{/each}
        {/snippet}
        <section>
          <h3>Account</h3>
          <DetailList rows={detailRows(selected)} stackAt={36} />
        </section>
        <section>
          <h3>
            Member of {selected.groups.length}
            {selected.groups.length === 1 ? "group" : "groups"}
          </h3>
          <div class="groups">
            {#each selected.groups as g}
              <button
                class="group"
                onclick={() => {
                  accounts.openGroup(g);
                  goto("/accounts/groups");
                }}>{g}</button
              >
            {:else}
              <p class="muted">None reported.</p>
            {/each}
          </div>
        </section>
      </SidePanel>
    {/if}
  {/snippet}
</AccountsShell>

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
  .name {
    display: inline-flex;
    align-items: center;
    gap: var(--s-2);
  }
  small {
    display: block;
    font-family: var(--font-ui);
  }
  .groups {
    display: flex;
    flex-wrap: wrap;
    gap: var(--s-2);
  }
  .group {
    padding: 3px var(--s-3);
    border: 1px solid var(--border);
    background: var(--surface-2);
    color: var(--text-2);
    font-family: var(--font-mono);
    font-size: 11px;
    cursor: pointer;
  }
  .group:hover {
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
