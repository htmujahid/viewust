import type { Detail } from "./common";

export interface Connection {
  kind: "ethernet" | "wifi" | "other";
  interface: string;
  link_label: string;
  signal: number | null;
  router_name: string;
  router_details: Detail[];
  connectivity: "full" | "limited" | "portal" | "none" | "unknown";
  internet_details: Detail[];
}
