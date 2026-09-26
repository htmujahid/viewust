import type { Detail } from "./common";

export interface Component {
  id: string;
  kind: string;
  name: string;
  subtitle: string | null;
  details: Detail[];
}

export interface SystemInfo {
  computer_name: string;
  components: Component[];
}

export interface MemoryModules {
  modules: Component[];
  slots: number;
}
