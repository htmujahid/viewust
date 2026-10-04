import { api } from "$lib/api/client";
import type {
  Accounts,
  Cgroups,
  DiskDevices,
  Filesystems,
  Namespaces,
  OsNetwork,
  ServiceSnapshot,
  Snapshot,
} from "$lib/api/types";

import { Loaded } from "./loaded.svelte";

/** The one-shot reads the OS dashboard needs from the other subsystems. */
class OsOverview {
  readonly processes = new Loaded<Snapshot>(() => api.processList());
  readonly namespaces = new Loaded<Namespaces>(() => api.namespaceList());
  readonly services = new Loaded<ServiceSnapshot>(() => api.serviceList());
  readonly accounts = new Loaded<Accounts>(() => api.accountList());
  readonly disks = new Loaded<DiskDevices>(() => api.diskDevices());
  readonly cgroups = new Loaded<Cgroups>(() => api.osCgroups());
  readonly network = new Loaded<OsNetwork>(() => api.osNetwork());
  readonly filesystems = new Loaded<Filesystems>(() => api.filesystemList());

  readonly all = [
    this.processes,
    this.namespaces,
    this.services,
    this.accounts,
    this.disks,
    this.cgroups,
    this.network,
    this.filesystems,
  ];

  ensure = () => this.all.forEach((l) => l.ensure());
  reload = () => this.all.forEach((l) => void l.load());
}

export const osOverview = new OsOverview();
