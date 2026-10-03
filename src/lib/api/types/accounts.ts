export interface UserRow {
  name: string;
  uid: string;
  primary_group: string;
  groups: string[];
  full_name: string | null;
  home: string | null;
  shell: string | null;
  kind: "root" | "system" | "regular";
  admin: boolean;
  can_login: boolean | null;
  current: boolean;
  processes: number;
  memory: number;
}

export interface GroupRow {
  name: string;
  gid: string;
  members: string[];
  kind: "admin" | "system" | "regular";
  admin: boolean;
}

export interface Accounts {
  platform: "linux" | "macos" | "windows" | "other";
  users: UserRow[];
  groups: GroupRow[];
}
