import type { EnvVar, KernelModule, PackageRow } from "$lib/api/types";

/** The most packages drawn at once; a search narrows thousands down to a readable list. */
export const PACKAGE_LIMIT = 500;

const words = (query: string) => query.trim().toLowerCase().split(/\s+/).filter(Boolean);
const hasAll = (text: string, terms: string[]) => terms.every((t) => text.includes(t));

export function filterModules(rows: readonly KernelModule[], query: string): KernelModule[] {
  const terms = words(query);
  return rows.filter((m) => hasAll(`${m.name} ${m.used_by.join(" ")}`.toLowerCase(), terms));
}

export function filterPackages(rows: readonly PackageRow[], query: string): PackageRow[] {
  const terms = words(query);
  return rows.filter((p) => hasAll(`${p.name} ${p.version} ${p.arch ?? ""}`.toLowerCase(), terms));
}

export function filterEnv(rows: readonly EnvVar[], query: string): EnvVar[] {
  const terms = words(query);
  return rows.filter((e) => hasAll(`${e.key} ${e.hidden ? "" : e.value}`.toLowerCase(), terms));
}

/** Two arch variants of one package (libc6 on amd64 and i386) must not share a row key. */
export const packageKey = (p: PackageRow) => `${p.name}:${p.arch ?? ""}`;

/** `PATH`-like values are lists; show them one entry per line. */
export const isPathList = (value: string) => value.includes(":") && value.startsWith("/");

export const bootedAgo = (bootTime: number, now = Date.now() / 1000): number =>
  Math.max(0, Math.floor(now - bootTime));
