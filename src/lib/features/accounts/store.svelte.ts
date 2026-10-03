import { api, errorMessage } from "$lib/api/client";
import type { Accounts, GroupRow, UserRow } from "$lib/api/types";
import { Poller } from "$lib/utils/poller";

import { filterGroups, filterUsers, type GroupFilter, type UserFilter } from "./logic";

class AccountsStore {
  snapshot = $state.raw<Accounts | null>(null);
  error = $state<string | null>(null);

  userSearch = $state("");
  userFilter = $state<UserFilter>("all");
  userSort = $state("name");
  userDesc = $state(false);
  selectedUser = $state<string | null>(null);

  groupSearch = $state("");
  groupFilter = $state<GroupFilter>("all");
  groupSort = $state("name");
  groupDesc = $state(false);
  selectedGroup = $state<string | null>(null);

  readonly #poller = new Poller(() => this.#refresh(), 10000);

  start = () => this.#poller.start();
  stop = () => this.#poller.stop();

  async #refresh() {
    try {
      this.snapshot = await api.accountList();
      this.error = null;
    } catch (e) {
      this.error = errorMessage(e);
    }
  }

  get users(): UserRow[] {
    return filterUsers(
      this.snapshot?.users ?? [],
      this.userFilter,
      this.userSearch,
      this.userSort,
      this.userDesc,
    );
  }

  get groups(): GroupRow[] {
    return filterGroups(
      this.snapshot?.groups ?? [],
      this.groupFilter,
      this.groupSearch,
      this.groupSort,
      this.groupDesc,
    );
  }

  sortUsers(key: string) {
    if (this.userSort === key) this.userDesc = !this.userDesc;
    else {
      this.userSort = key;
      this.userDesc = key === "processes" || key === "memory" || key === "groups";
    }
  }

  sortGroups(key: string) {
    if (this.groupSort === key) this.groupDesc = !this.groupDesc;
    else {
      this.groupSort = key;
      this.groupDesc = key === "members";
    }
  }

  countUsers = (f: UserFilter) =>
    filterUsers(this.snapshot?.users ?? [], f, "", "name", false).length;
  countGroups = (f: GroupFilter) =>
    filterGroups(this.snapshot?.groups ?? [], f, "", "name", false).length;

  openGroup(name: string) {
    this.groupSearch = "";
    this.groupFilter = "all";
    this.selectedGroup = name;
  }

  openUser(name: string) {
    this.userSearch = "";
    this.userFilter = "all";
    this.selectedUser = name;
  }
}

export const accounts = new AccountsStore();
