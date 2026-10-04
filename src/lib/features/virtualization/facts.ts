import type { Namespaces, OsNetwork, VirtOverview, VirtSlice } from "$lib/api/types";
import type { Fact } from "$lib/features/os/subsystems";
import { formatBytes } from "$lib/utils/format";

const fact = (label: string, value: string, tone?: Fact["tone"]): Fact => ({ label, value, tone });

export function hostFacts(v: VirtOverview | null): Fact[] {
  if (!v) return [];
  return [
    v.cpu
      ? fact("Processor support", v.cpu, "ok")
      : fact("Processor support", "None found", "warn"),
    fact("/dev/kvm", v.kvm_device ? "Present" : "Missing", v.kvm_device ? "ok" : "warn"),
    ...(v.kvm_module ? [fact("KVM module", v.kvm_module)] : []),
    ...(v.nested !== null ? [fact("Nested guests", v.nested ? "Allowed" : "Off")] : []),
  ];
}

export function selfFacts(v: VirtOverview | null): Fact[] {
  if (!v) return [];
  return [
    fact(
      "This machine",
      v.inside_vm ? `A virtual machine (${v.inside_vm})` : "Physical",
      v.inside_vm ? "warn" : undefined,
    ),
    ...(v.inside_container
      ? [fact("Container", `Inside ${v.inside_container}`, "warn")]
      : [fact("Container", "Not inside one")]),
    fact(
      "Can host guests",
      v.cpu && v.kvm_device ? `Yes (${v.cpu} + KVM)` : "Containers only",
      v.cpu && v.kvm_device ? "ok" : undefined,
    ),
  ];
}

export function guestStorageFacts(v: VirtOverview | null): Fact[] {
  if (!v) return [];
  return [
    fact("Overlay mounts", String(v.overlay_mounts), v.overlay_mounts ? "ok" : undefined),
    ...v.docker_df.map((d) =>
      fact(d.kind, `${d.count} · ${d.size} (${d.reclaimable} reclaimable)`),
    ),
    ...(v.docker_df.length === 0 ? [fact("Container storage", "The runtime isn't answering")] : []),
  ];
}

export function guestCgroupFacts(v: VirtOverview | null): Fact[] {
  if (!v) return [];
  if (v.slices.length === 0) {
    return [
      fact("Guest groups", "None found"),
      fact("Where they appear", "machine.slice and docker-*.scope"),
      fact("Right now", "No container or VM is being charged"),
    ];
  }
  const total = (pick: (s: VirtSlice) => number | null) =>
    v.slices.reduce((n, s) => n + (pick(s) ?? 0), 0);
  const perSlice = v.slices.map((s) =>
    fact(
      s.name,
      [
        `${s.groups} group${s.groups === 1 ? "" : "s"}`,
        s.pids !== null ? `${s.pids} processes` : null,
        s.memory !== null ? formatBytes(s.memory) : null,
      ]
        .filter(Boolean)
        .join(" · "),
    ),
  );
  return [
    ...perSlice,
    fact("Guest processes in all", String(total((s) => s.pids))),
    fact("Memory charged in all", formatBytes(total((s) => s.memory))),
  ];
}

export function isolationFacts(n: Namespaces | null): Fact[] {
  if (!n) return [];
  const isolated = n.namespaces.filter((x) => !x.current);
  const kinds = new Set(isolated.map((x) => x.kind));
  return [
    fact("Isolated namespaces", String(isolated.length), isolated.length ? "ok" : undefined),
    fact("Kinds in use", kinds.size ? [...kinds].join(", ") : "none"),
    fact("Shared with the host", String(n.namespaces.length - isolated.length)),
    fact("Programs read", `${n.inspected} of ${n.total}`),
  ];
}

export function guestNetworkFacts(net: OsNetwork | null, v: VirtOverview | null): Fact[] {
  if (!net) return [];
  const bridges = net.interfaces.filter((i) => i.kind === "bridge");
  const virtual = net.interfaces.filter((i) => i.kind === "virtual");
  return [
    fact("Bridges", bridges.length ? bridges.map((b) => b.name).join(", ") : "none"),
    fact("Virtual interfaces", String(virtual.length)),
    ...(v
      ? [
          fact(
            "Packet forwarding",
            v.ip_forward ? "On — guests can reach out" : "Off",
            v.ip_forward ? "ok" : undefined,
          ),
        ]
      : []),
    fact("Host uplink", net.default_route ?? "No default route"),
  ];
}
