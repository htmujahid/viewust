import { api, errorMessage } from "$lib/api/client";

import { Loaded } from "./loaded.svelte";
import type {
  EnvVar,
  KernelModule,
  ModuleInfo,
  OsMemory,
  OsSecurity,
  OsSummary,
  Packages,
} from "$lib/api/types";

class OsStore {
  readonly summary = new Loaded<OsSummary>(() => api.osSummary());
  readonly modules = new Loaded<KernelModule[]>(() => api.kernelModules());
  readonly packages = new Loaded<Packages>(() => api.osPackages());
  readonly environment = new Loaded<EnvVar[]>(() => api.osEnvironment());
  readonly memory = new Loaded<OsMemory>(() => api.osMemory());
  readonly security = new Loaded<OsSecurity>(() => api.osSecurity());

  moduleSearch = $state("");
  moduleSort = $state("size");
  moduleDesc = $state(true);
  selectedModule = $state<string | null>(null);
  moduleInfo = $state.raw<ModuleInfo | null>(null);

  packageSearch = $state("");
  envSearch = $state("");

  async selectModule(name: string | null) {
    this.selectedModule = name;
    this.moduleInfo = null;
    if (name === null) return;
    try {
      this.moduleInfo = await api.kernelModuleInfo(name);
    } catch (e) {
      this.moduleInfo = { name, found: false, details: [] };
      this.modules.error = errorMessage(e);
    }
  }

  sortModules(key: string) {
    if (this.moduleSort === key) this.moduleDesc = !this.moduleDesc;
    else {
      this.moduleSort = key;
      this.moduleDesc = key !== "name";
    }
  }
}

export const os = new OsStore();
