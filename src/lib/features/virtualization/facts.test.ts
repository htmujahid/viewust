import { describe, expect, it } from "vitest";

import { fixtures } from "$lib/api/mock";

import { guestCgroupFacts, guestStorageFacts, hostFacts, isolationFacts, selfFacts } from "./facts";

const virt = fixtures.virtOverviewMock();
const by = (facts: { label: string; value: string; tone?: string }[], label: string) =>
  facts.find((f) => f.label === label);

describe("virtualization facts", () => {
  it("says whether the hardware can host guests", () => {
    expect(by(hostFacts(virt), "Processor support")).toMatchObject({
      value: "Intel VT-x",
      tone: "ok",
    });
    expect(by(hostFacts({ ...virt, cpu: null, kvm_device: false }), "/dev/kvm")?.tone).toBe("warn");
    expect(hostFacts(null)).toEqual([]);
  });

  it("flags running inside a VM or container", () => {
    expect(by(selfFacts(virt), "This machine")?.value).toBe("Physical");
    expect(by(selfFacts({ ...virt, inside_vm: "kvm" }), "This machine")?.tone).toBe("warn");
  });

  it("shows guest storage and cgroup slices", () => {
    expect(by(guestStorageFacts(virt), "Images")?.value).toContain("1.809GB");
    const cg = guestCgroupFacts(virt);
    expect(by(cg, "docker containers")?.value).toContain("23 processes");
    expect(guestCgroupFacts({ ...virt, slices: [] })[0].value).toBe("None found");
  });

  it("counts the walls around guests", () => {
    const f = isolationFacts(fixtures.namespaceList());
    expect(Number(by(f, "Isolated namespaces")?.value)).toBeGreaterThan(0);
  });
});
