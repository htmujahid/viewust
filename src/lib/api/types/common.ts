/** One row in a details panel. Rows sharing a `section` are shown together. */
export interface Detail {
  section: string;
  label: string;
  value: string;
}
