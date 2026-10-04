import { api, errorMessage } from "$lib/api/client";
import type { EnvVar, KernelModule, ModuleInfo, OsSummary, Packages } from "$lib/api/types";

/** One list that is read the first time its tab is opened, and again on request. */
class Loaded<T> {
  data = $state.raw<T | null>(null);
  error = $state<string | null>(null);
  loading = $state(false);

  constructor(private readonly read: () => Promise<T>) {}

  load = async () => {
    this.loading = true;
    try {
      this.data = await this.read();
      this.error = null;
    } catch (e) {
      this.error = errorMessage(e);
    } finally {
      this.loading = false;
    }
  };

  ensure = () => {
    if (this.data === null && !this.loading) void this.load();
  };
}

class OsStore {
  readonly summary = new Loaded<OsSummary>(() => api.osSummary());
  readonly modules = new Loaded<KernelModule[]>(() => api.kernelModules());
  readonly packages = new Loaded<Packages>(() => api.osPackages());
  readonly environment = new Loaded<EnvVar[]>(() => api.osEnvironment());

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
