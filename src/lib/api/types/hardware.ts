import type { Detail } from "./common";
import type { Connection } from "./connection";

export interface Peripheral {
  id: string;
  via: string | null;
  wireless: boolean;
  name: string;
  manufacturer: string | null;
  kind: string;
  connection: string;
  vendor_id: string;
  product_id: string;
  serial_number: string | null;
  details: Detail[];
}

export interface Display {
  name: string;
  connector: string | null;
  width_cm: number | null;
  width: number;
  height: number;
  scale_factor: number;
  primary: boolean;
  details: Detail[];
}

export interface HardwareInfo {
  computer_name: string;
  computer_details: Detail[];
  connection: Connection | null;
  peripherals: Peripheral[];
  displays: Display[];
}
