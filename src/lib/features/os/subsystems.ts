import type {
  Accounts,
  Cgroups,
  DiskDevices,
  EnvVar,
  Filesystems,
  KernelModule,
  Namespaces,
  OsMemory,
  OsNetwork,
  OsSecurity,
  OsSummary,
  Packages,
  ServiceSnapshot,
  Snapshot,
} from "$lib/api/types";
import { formatBytes, formatCount } from "$lib/utils/format";

export interface Fact {
  label: string;
  value: string;
  tone?: "ok" | "warn" | "danger";
}

const fact = (label: string, value: string, tone?: Fact["tone"]): Fact => ({ label, value, tone });

/** Picks a handful of rows out of the one summary, for a card. */
export function summaryFacts(
  s: OsSummary | null,
  wanted: readonly [section: string, label: string][],
): Fact[] {
  if (!s) return [];
  return wanted.flatMap(([section, label]) => {
    const row = s.details.find((d) => d.section === section && d.label === label);
    return row ? [fact(row.label, row.value)] : [];
  });
}

/** The bootloader's name alone; its full boot-chain row carries the explanation. */
export function bootloaderFact(s: OsSummary | null): Fact[] {
  const row = s?.details.find((d) => d.section === "Boot chain" && d.label.endsWith("Bootloader"));
  const name = row?.value.split(" — ")[0];
  return name ? [fact("Bootloader", name)] : [];
}

export function kernelFacts(s: OsSummary | null, modules: KernelModule[] | null): Fact[] {
  if (!s) return [];
  const tainted = s.details.find((d) => d.label === "Tainted")?.value;
  return [
    fact("Release", s.kernel),
    ...(modules ? [fact("Drivers loaded", String(modules.length))] : []),
    ...(tainted ? [fact("Tainted", tainted, tainted === "No" ? undefined : "warn")] : []),
  ];
}

export function processFacts(s: Snapshot | null): Fact[] {
  if (!s) return [];
  const o = s.overview;
  return [
    fact("Processes", String(o.processes)),
    fact("Running now", String(o.running)),
    fact("Threads", formatCount(o.threads)),
    fact("Load", o.load.map((l) => l.toFixed(2)).join(" · ")),
  ];
}

export function namespaceFacts(n: Namespaces | null): Fact[] {
  if (!n) return [];
  const kinds = new Set(n.namespaces.map((x) => x.kind));
  return [
    fact("Namespaces", String(n.namespaces.length)),
    fact("Types in use", `${kinds.size} of 8`),
    fact("Programs read", `${n.inspected} of ${n.total}`),
  ];
}

export function cgroupFacts(c: Cgroups | null): Fact[] {
  if (!c) return [];
  const head = c.controllers.slice(0, 4).join(", ");
  const more = c.controllers.length - 4;
  return [
    fact("Version", c.version),
    fact("Groups", c.capped ? `over ${c.groups}` : String(c.groups)),
    ...(c.controllers.length ? [fact("Controllers", more > 0 ? `${head} +${more}` : head)] : []),
  ];
}

export function memoryFacts(m: OsMemory | null): Fact[] {
  if (!m) return [];
  const share = m.total > 0 ? (m.used / m.total) * 100 : 0;
  return [
    fact(
      "In use",
      `${formatBytes(m.used)} of ${formatBytes(m.total)} (${share.toFixed(0)}%)`,
      share >= 90 ? "danger" : share >= 75 ? "warn" : undefined,
    ),
    fact("Available", formatBytes(m.available)),
    fact(
      "Swap",
      m.swap_total ? `${formatBytes(m.swap_used)} of ${formatBytes(m.swap_total)}` : "None",
    ),
  ];
}

export function filesystemFacts(d: DiskDevices | null): Fact[] {
  if (!d) return [];
  const o = d.overview;
  return [
    fact("Drives", `${o.disks} · ${formatBytes(o.capacity)}`),
    fact("Used", formatBytes(o.used)),
    fact("Free", formatBytes(o.available)),
    ...(o.unmounted
      ? [fact("Not mounted", `${o.unmounted} filesystems`, "warn")]
      : [fact("Mounted", `${o.mounted} filesystems`)]),
  ];
}

export function serviceFacts(s: ServiceSnapshot | null): Fact[] {
  if (!s) return [];
  const o = s.overview;
  return [
    fact("Installed", String(o.total)),
    fact("Running", String(o.running), "ok"),
    fact("Failed", String(o.failed), o.failed > 0 ? "danger" : undefined),
    fact("Start at boot", String(o.enabled)),
  ];
}

export function mountFacts(f: Filesystems | null): Fact[] {
  if (!f) return [];
  const biggest = f.rows.find((r) => !r.pseudo);
  return [
    fact("Mounted", String(f.mounted)),
    fact("With real space", String(f.real)),
    ...(biggest ? [fact("Largest", `${biggest.mount} · ${formatBytes(biggest.size ?? 0)}`)] : []),
  ];
}

export function networkFacts(n: OsNetwork | null): Fact[] {
  if (!n) return [];
  return [
    fact("Interfaces up", `${n.up} of ${n.total}`, n.up === 0 ? "danger" : "ok"),
    ...(n.default_route
      ? [fact("Internet", n.default_route)]
      : [fact("Internet", "No default route", "warn")]),
    fact("DNS servers", String(n.dns.length), n.dns.length === 0 ? "warn" : undefined),
    fact("Listening TCP ports", String(n.listening_tcp.length)),
    fact("Connections", String(n.established)),
  ];
}

export function userFacts(a: Accounts | null): Fact[] {
  if (!a) return [];
  const people = a.users.filter((u) => u.kind === "regular").length;
  const admins = a.users.filter((u) => u.admin).length;
  return [
    fact("Accounts", String(a.users.length)),
    fact("People", String(people)),
    fact("Administrators", String(admins), admins > 1 ? "warn" : undefined),
    fact("Can sign in", String(a.users.filter((u) => u.can_login === true).length)),
  ];
}

export function groupFacts(a: Accounts | null): Fact[] {
  if (!a) return [];
  const admin = a.groups.filter((g) => g.admin).length;
  return [
    fact("Groups", String(a.groups.length)),
    fact("Admin groups", String(admin), admin > 0 ? "warn" : undefined),
    fact("With members", String(a.groups.filter((g) => g.members.length > 0).length)),
  ];
}

export function securityFacts(s: OsSecurity | null): Fact[] {
  if (!s) return [];
  return [
    fact(
      "Access control",
      s.apparmor ? `AppArmor ${s.apparmor.toLowerCase()}` : (s.selinux ?? "None"),
      s.apparmor === "Enabled" || s.selinux === "Enforcing" ? "ok" : "warn",
    ),
    ...(s.secure_boot
      ? [fact("Secure Boot", s.secure_boot, s.secure_boot === "Enabled" ? "ok" : "warn")]
      : []),
    ...(s.lockdown ? [fact("Lockdown", s.lockdown)] : []),
  ];
}

export function packageFacts(p: Packages | null): Fact[] {
  if (!p) return [];
  if (p.manager === null) return [fact("Package manager", "None found", "warn")];
  return [fact("Manager", p.manager), fact("Installed", String(p.packages.length))];
}

export function envFacts(e: EnvVar[] | null): Fact[] {
  if (!e) return [];
  const hidden = e.filter((v) => v.hidden).length;
  return [
    fact("Variables", String(e.length)),
    ...(hidden ? [fact("Hidden secrets", String(hidden))] : []),
  ];
}
