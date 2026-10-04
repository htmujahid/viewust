export type Severity = "ok" | "warn" | "danger";

export interface HealthCheck {
  severity: Severity;
  title: string;
  detail: string;
  /** Where to look closer, when there is somewhere */
  link: string | null;
}

export interface HealthReport {
  problems: number;
  warnings: number;
  fine: number;
  /** Worst first */
  checks: HealthCheck[];
}
