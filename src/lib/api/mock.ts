import { mockIPC } from "@tauri-apps/api/mocks";

import type {
  Component,
  Connection,
  Detail,
  EnvVar,
  KernelModule,
  ModuleInfo,
  Cgroups,
  OsMemory,
  OsNetwork,
  OsSecurity,
  OsSummary,
  PackageRow,
  Packages,
  DirectoryUsage,
  Disk,
  DiskDevices,
  Filesystems,
  UsageEntry,
  DiskVolume,
  HardwareInfo,
  MemoryModules,
  Peripheral,
  Accounts,
  Namespaces,
  ProcessDetail,
  Sample,
  ServiceDetail,
  ServiceSnapshot,
  Snapshot,
  SystemInfo,
} from "$lib/api/types";

const row = (section: string, label: string, value: string): Detail => ({ section, label, value });

const usbFacts = (product: string, extra: Detail[] = []): Detail[] => [
  row("Device", "Product", product),
  row("Identification", "Vendor ID", "0x046d"),
  row("Identification", "Product ID", "0x082d"),
  row("Connection", "Speed", "High speed · 480 Mbit/s"),
  row("Power", "Maximum draw", "500mA"),
  ...extra,
];

const peripheral = (
  p: Partial<Peripheral> & Pick<Peripheral, "id" | "name" | "kind">,
): Peripheral => ({
  via: null,
  wireless: false,
  manufacturer: null,
  connection: "USB 2.0",
  vendor_id: "046d",
  product_id: "082d",
  serial_number: null,
  details: usbFacts(p.name),
  ...p,
});

const peripherals: Peripheral[] = [
  peripheral({ id: "001-2", name: "USB Receiver", kind: "wireless", manufacturer: "Logitech" }),
  peripheral({
    id: "001-2:keyboard",
    name: "USB Receiver",
    kind: "keyboard",
    via: "001-2",
    wireless: true,
  }),
  peripheral({
    id: "001-2:mouse",
    name: "USB Receiver",
    kind: "mouse",
    via: "001-2",
    wireless: true,
  }),
  peripheral({
    id: "001-4:webcam",
    name: "HD Pro Webcam C920",
    kind: "webcam",
    manufacturer: "Logitech",
  }),
  peripheral({
    id: "jack-0-front-headphone",
    name: "Front Headphones",
    kind: "audio",
    connection: "3.5 mm jack",
  }),
  peripheral({
    id: "jack-0-front-mic",
    name: "Front microphone",
    kind: "microphone",
    connection: "3.5 mm jack",
  }),
  peripheral({ id: "001-6:storage", name: "Ultra Fit", kind: "storage", manufacturer: "SanDisk" }),
  peripheral({ id: "001-7:printer", name: "LaserJet Pro", kind: "printer", manufacturer: "HP" }),
];

const connections: Record<string, Connection | null> = {
  wired: {
    kind: "ethernet",
    interface: "enp3s0",
    link_label: "Ethernet · 1 Gbit/s",
    signal: null,
    router_name: "Router",
    router_details: [
      row("Router", "Address (gateway)", "192.168.1.1"),
      row("Cable", "Link speed", "1000 Mbit/s"),
    ],
    connectivity: "full",
    internet_details: [row("Internet", "Status", "Online (confirmed by NetworkManager)")],
  },
  wifi: {
    kind: "wifi",
    interface: "wlan0",
    link_label: "Wi-Fi · 5 GHz · 866.7 Mbit/s",
    signal: 82,
    router_name: "Home Net",
    router_details: [row("Wi-Fi", "Signal", "-59 dBm · good"), row("Wi-Fi", "Channel", "36")],
    connectivity: "full",
    internet_details: [row("Internet", "Status", "Online (confirmed by NetworkManager)")],
  },
  offline: {
    kind: "ethernet",
    interface: "enp3s0",
    link_label: "Ethernet · 100 Mbit/s",
    signal: null,
    router_name: "Router",
    router_details: [],
    connectivity: "none",
    internet_details: [row("Internet", "Status", "Not connected")],
  },
};

const hardware = (net: string): HardwareInfo => ({
  computer_name: "talha-MS-7E02",
  computer_details: [
    row("System", "Model", "MS-7E02"),
    row("Processor", "Model", "12th Gen Intel i7-12700KF"),
  ],
  connection: connections[net] ?? connections.wired,
  peripherals,
  displays: [
    {
      name: "P27FBB-RGGL",
      connector: "DP-3",
      width_cm: 60,
      width: 1920,
      height: 1080,
      scale_factor: 1,
      primary: true,
      details: [row("Monitor", "Manufacturer", "Xiaomi (XMI)")],
    },
    {
      name: "S24E450",
      connector: "DVI-D-1",
      width_cm: 53,
      width: 1920,
      height: 1080,
      scale_factor: 1,
      primary: false,
      details: [row("Monitor", "Manufacturer", "Samsung (SAM)")],
    },
  ],
});

const part = (
  id: string,
  kind: string,
  name: string,
  subtitle: string,
  details: Detail[] = [],
): Component => ({
  id,
  kind,
  name,
  subtitle,
  details: details.length ? details : [row("Info", "Name", name)],
});

const components: Component[] = [
  part("sys:board", "board", "PRO B760M-P (MS-7E02)", "Micro-Star International Co., Ltd.", [
    row("Motherboard", "Model", "PRO B760M-P (MS-7E02)"),
  ]),
  part("sys:cpu", "cpu", "12th Gen Intel(R) Core(TM) i7-12700KF", "12 cores · 20 threads", [
    row("Power and heat", "Sustained power limit (PL1)", "135 W"),
  ]),
  part("sys:ram", "ram", "System memory", "30.7 GiB", [
    row("Memory", "Usable", "30.7 GiB"),
    row("Memory modules", "Status", "Not read yet."),
  ]),
  part("sys:gpu:card0", "gpu", "NVIDIA GeForce GTX 1060 3GB", "3.0 GiB · NVIDIA Corporation"),
  part("sys:disk:sda", "hdd", "ST1000LM035-1RK172", "1.0 TB · SATA"),
  part("sys:disk:sdb", "hdd", "ST2000DM006-2DM164", "2.0 TB · SATA"),
  part("sys:disk:sdc", "ssd", "Samsung SSD 860 EVO 500GB", "500 GB · SATA"),
  part("sys:nic:enp3s0", "nic", "RTL8111/8168 Gigabit Ethernet", "enp3s0 · Ethernet"),
  part("sys:audio:0", "soundcard", "Realtek ALC897", "Raptor Lake HD Audio Controller"),
  part("sys:psu", "psu", "Power supply", "Not reported by hardware"),
];

const memoryModules: MemoryModules = {
  slots: 4,
  modules: [0, 1].map((i) =>
    part(
      `sys:ram:${i}`,
      "ram",
      "Kingston KF548C38-16",
      `DIMM_${i ? "B" : "A"}1 · 16 GB · 4800 MT/s`,
      [row("Module", "Capacity", "16 GB")],
    ),
  ),
};

const processNames = [
  "firefox",
  "Isolated Web Co",
  "chrome",
  "code",
  "gnome-shell",
  "Xorg",
  "pipewire",
  "systemd",
  "dbus-daemon",
  "cargo",
  "node",
  "kworker/0:1",
];

const processList = (tick: number): Snapshot => ({
  overview: {
    processes: processNames.length,
    running: 1,
    threads: 2401,
    cpu: 10 + (tick % 7),
    cpu_count: 20,
    load: [1.43, 1.23, 1.22],
    memory_total: 32947e6,
    memory_used: 9569e6,
    memory_available: 21500e6,
    swap_total: 16000e6,
    swap_used: 0,
    uptime: 54191,
  },
  processes: processNames.map((name, i) => ({
    pid: 1000 + i * 37,
    parent: i ? 1000 : 1,
    name,
    user: i % 5 === 0 ? "root" : "talha",
    cpu: Math.max(0, (i === 0 ? 31 : 60 / (i + 1)) + Math.sin(tick + i) * 2),
    memory: Math.round(1284e6 / (i * 1.7 + 1)),
    virtual_memory: Math.round(12811e6 / (i * 0.9 + 1)),
    threads: Math.max(1, 116 - i * 4),
    state: i === 0 ? "Running" : "Sleeping",
    kernel: name.startsWith("k"),
    run_time: 5000 + i * 100,
  })),
});

const processDetail = (pid: number): ProcessDetail => ({
  pid,
  running: true,
  restricted: pid % 5 === 0,
  memory: {
    requested: 12811e6,
    peak_requested: 36600e6,
    resident: 1284e6,
    peak_resident: 2300e6,
    anonymous: 780e6,
    file_backed: 390e6,
    shared: 113e6,
    swapped: 0,
    data: 900e6,
    stack: 8e6,
    code: 20e6,
    libraries: 400e6,
    page_tables: 12e6,
    locked: 0,
    proportional: 1049e6,
    private: 900e6,
    shared_pages: 380e6,
  },
  regions: [
    { name: "[anon:jemalloc]", size: 895e6, resident: 689e6 },
    { name: "/snap/firefox/libxul.so", size: 172e6, resident: 123e6 },
  ],
  children: [{ pid: 8850, name: "Isolated Web Co", memory: 1130e6 }],
  details: [
    row("Process", "Name", "firefox"),
    row("Process", "ID (PID)", String(pid)),
    row("Command", "Command line", "/snap/firefox/8929/usr/lib/firefox/firefox -new-window"),
    row("CPU", "CPU time used", "1h 12m 3s"),
    row("Memory requests", "Asked for (virtual)", "12.5 GiB"),
    row("Files", "Open files and sockets", "4752"),
  ],
});

const accountList = (): Accounts => {
  const user = (
    name: string,
    uid: string,
    kind: "root" | "system" | "regular",
    groups: string[],
    extra: Partial<Accounts["users"][number]> = {},
  ): Accounts["users"][number] => ({
    name,
    uid,
    primary_group: groups[0] ?? name,
    groups,
    full_name: null,
    home: kind === "regular" ? `/home/${name}` : "/nonexistent",
    shell: kind === "regular" ? "/bin/bash" : "/usr/sbin/nologin",
    kind,
    admin: groups.some((g) => ["sudo", "wheel", "admin", "root"].includes(g)),
    can_login: kind === "regular" || kind === "root",
    current: false,
    processes: 0,
    memory: 0,
    ...extra,
  });
  const users = [
    user("talha", "1000", "regular", ["talha", "sudo", "docker", "adm", "users"], {
      full_name: "Talha M",
      current: true,
      processes: 199,
      memory: 19.8e9,
    }),
    user("guest", "1001", "regular", ["guest", "users"], {
      full_name: "Guest",
      shell: "/bin/zsh",
      processes: 0,
    }),
    user("root", "0", "root", ["root"], {
      home: "/root",
      shell: "/bin/bash",
      processes: 392,
      memory: 2.3e9,
    }),
    user("www-data", "33", "system", ["www-data"], { processes: 3, memory: 41e6 }),
    user("daemon", "1", "system", ["daemon"], { processes: 1, memory: 2e6 }),
    user("systemd-resolve", "991", "system", ["systemd-resolve"], { processes: 1, memory: 8e6 }),
    user("nobody", "65534", "system", ["nogroup"]),
  ];
  const memberships = new Map<string, string[]>();
  for (const u of users)
    for (const g of u.groups) memberships.set(g, [...(memberships.get(g) ?? []), u.name]);
  const gid: Record<string, string> = {
    root: "0",
    daemon: "1",
    adm: "4",
    sudo: "27",
    "www-data": "33",
    users: "100",
    docker: "999",
    talha: "1000",
    guest: "1001",
    "systemd-resolve": "991",
    nogroup: "65534",
    cdrom: "24",
  };
  const adminNames = ["sudo", "root", "adm"];
  const groups = Object.entries(gid).map(([name, id]) => ({
    name,
    gid: id,
    members: memberships.get(name) ?? [],
    kind:
      adminNames.includes(name) && name !== "adm"
        ? ("admin" as const)
        : Number(id) >= 1000 && Number(id) < 65534
          ? ("regular" as const)
          : ("system" as const),
    admin: adminNames.includes(name) && name !== "adm",
  }));
  return { users, groups };
};

const namespaceList = (): Namespaces => {
  const host: Record<string, number> = {
    pid: 4026531836,
    net: 4026531833,
    mnt: 4026531832,
    user: 4026531837,
    uts: 4026531838,
    ipc: 4026531839,
    cgroup: 4026531835,
    time: 4026531834,
  };
  const proc = (pid: number, name: string, user = "talha") => ({ pid, name, user });
  const rows: Namespaces["namespaces"] = Object.entries(host).map(([kind, id], i) => ({
    kind,
    id,
    processes: 150 + i * 6,
    sample: [proc(1148, "gnome-shell"), proc(1222, "pipewire"), proc(1296, "dbus-daemon")],
    current: true,
  }));
  const isolated = (
    kind: string,
    id: number,
    processes: number,
    sample: ReturnType<typeof proc>[],
  ) => rows.push({ kind, id, processes, sample, current: false });
  isolated("user", 4026532801, 6, [proc(8850, "firefox"), proc(8901, "Isolated Web Co")]);
  isolated("pid", 4026532802, 6, [proc(8850, "firefox"), proc(8901, "Isolated Web Co")]);
  isolated("net", 4026532803, 2, [proc(8901, "Isolated Web Co")]);
  isolated("mnt", 4026532804, 6, [proc(8850, "firefox")]);
  isolated("pid", 4026532900, 4, [proc(9100, "nginx", "root"), proc(9112, "nginx", "www-data")]);
  isolated("net", 4026532902, 4, [proc(9100, "nginx", "root")]);
  isolated("mnt", 4026532901, 4, [proc(9100, "nginx", "root")]);
  isolated("uts", 4026532903, 4, [proc(9100, "nginx", "root")]);
  isolated("ipc", 4026532904, 4, [proc(9100, "nginx", "root")]);
  return {
    note: "Read 191 of 534 processes. The rest belong to other users, and only an administrator can look inside them.",
    inspected: 191,
    total: 534,
    namespaces: rows,
  };
};

const GB = 1e9;

const vol = (
  path: string,
  size: number,
  role: DiskVolume["role"],
  extra: Partial<DiskVolume> = {},
): DiskVolume => ({
  path,
  name: path.split("/").pop() ?? path,
  size,
  fstype: role === "filesystem" ? "ext4" : role === "swap" ? "swap" : null,
  label: null,
  mount: null,
  used: null,
  available: null,
  role,
  mountable: false,
  children: [],
  ...extra,
});

const mounted = (
  path: string,
  size: number,
  used: number,
  mount: string,
  extra: Partial<DiskVolume> = {},
) => vol(path, size, "filesystem", { mount, used, available: size - used, ...extra });

/** Volumes the person has mounted during this session. */
const mountedHere = new Map<string, string>();

const diskDevices = (): DiskDevices => {
  const disks: Disk[] = [
    {
      path: "/dev/nvme0n1",
      name: "nvme0n1",
      model: "Samsung SSD 980 PRO 1TB",
      size: 1000 * GB,
      kind: "nvme",
      removable: false,
      volumes: [
        mounted("/dev/nvme0n1p1", 1.1 * GB, 0.006 * GB, "/boot/efi", { fstype: "vfat" }),
        vol("/dev/nvme0n1p2", 16 * GB, "swap"),
        vol("/dev/nvme0n1p3", 980 * GB, "container", {
          fstype: "crypto_LUKS",
          children: [
            mounted("/dev/mapper/cryptroot", 970 * GB, 217 * GB, "/", { fstype: "btrfs" }),
          ],
        }),
      ],
    },
    {
      path: "/dev/sda",
      name: "sda",
      model: "ST1000LM035-1RK172",
      size: 1000 * GB,
      kind: "hdd",
      removable: false,
      volumes: [
        vol("/dev/sda1", 0.5 * GB, "filesystem", { fstype: "vfat", mountable: true }),
        vol("/dev/sda2", 999 * GB, "filesystem", {
          fstype: "ntfs",
          label: "New Volume",
          mountable: true,
        }),
      ],
    },
    {
      path: "/dev/sdb",
      name: "sdb",
      model: "ST2000DM006-2DM164",
      size: 2000 * GB,
      kind: "hdd",
      removable: false,
      volumes: [
        vol("/dev/sdb1", 0.13 * GB, "empty"),
        mounted("/dev/sdb2", 1999 * GB, 1210 * GB, "/mnt/archive", {
          fstype: "ntfs",
          label: "Local Disk",
        }),
      ],
    },
    {
      path: "/dev/sdc",
      name: "sdc",
      model: "Flash Drive",
      size: 62 * GB,
      kind: "usb",
      removable: true,
      volumes: [
        mounted("/dev/sdc1", 62 * GB, 11 * GB, "/media/talha/STICK", {
          fstype: "exfat",
          label: "STICK",
        }),
      ],
    },
    {
      path: "network",
      name: "Network shares",
      model: null,
      size: 9200 * GB,
      kind: "network",
      removable: false,
      volumes: [
        mounted("/mnt/nas", 7200 * GB, 5100 * GB, "/mnt/nas", {
          name: "nas.local:/volume1/media",
          fstype: "nfs4",
        }),
        mounted("/mnt/work", 2000 * GB, 1980 * GB, "/mnt/work", {
          name: "//fileserver/work",
          fstype: "cifs",
        }),
      ],
    },
  ];
  const local = disks.filter((d) => d.kind !== "network");
  const all: DiskVolume[] = [];
  const walk = (vs: DiskVolume[]) => vs.forEach((v) => (all.push(v), walk(v.children)));
  local.forEach((d) => walk(d.volumes));
  for (const v of all) {
    const at = mountedHere.get(v.path);
    if (at)
      Object.assign(v, {
        mount: at,
        mountable: false,
        used: v.size * 0.4,
        available: v.size * 0.6,
      });
  }
  const fs = all.filter((v) => v.role === "filesystem");
  return {
    overview: {
      capacity: local.reduce((n, d) => n + d.size, 0),
      used: fs.reduce((n, v) => n + (v.used ?? 0), 0),
      available: fs.reduce((n, v) => n + (v.available ?? 0), 0),
      disks: local.length,
      mounted: fs.filter((v) => v.mount).length,
      unmounted: fs.filter((v) => !v.mount).length,
    },
    disks,
  };
};

/** Where a mounted device's files are, for measuring. */
const mountPoints = (): [string, number][] =>
  diskDevices().disks.flatMap((d) =>
    d.volumes.flatMap(function flat(v): [string, number][] {
      return [
        ...(v.mount ? ([[v.mount, v.used ?? 0]] as [string, number][]) : []),
        ...v.children.flatMap(flat),
      ];
    }),
  );

const treeNames: [string, "dir" | "file"][] = [
  ["Documents", "dir"],
  ["Downloads", "dir"],
  [".cache", "dir"],
  ["projects", "dir"],
  ["Videos", "dir"],
  ["node_modules", "dir"],
  ["Pictures", "dir"],
  ["backup.tar.gz", "file"],
  ["notes.txt", "file"],
];
const weights = [0.38, 0.22, 0.14, 0.09, 0.06, 0.04, 0.03, 0.02, 0.01];
const measured = new Map<string, number>();

/** A believable folder tree: sizes shrink down the levels and each folder's children add up to it. */
const directoryUsage = (path: string): DirectoryUsage => {
  const mount = mountPoints().find((m) => m[0] === path);
  const total = measured.get(path) ?? (mount ? mount[1] : 4e9);
  const seed = [...path].reduce((n, c) => n + c.charCodeAt(0), 0);
  const depth = path.split("/").filter(Boolean).length;
  const count = depth >= 5 ? 0 : 5 + (seed % 4);
  const names = [...treeNames.slice(seed % 2), ...treeNames].slice(0, count);
  const entries: UsageEntry[] = names.map(([name, kind], i) => {
    const size = Math.round(total * 0.92 * weights[i] * (kind === "file" ? 0.5 : 1));
    const child = `${path === "/" ? "" : path}/${name}`;
    if (kind === "dir") measured.set(child, size);
    return {
      name,
      path: child,
      kind,
      size,
      files: kind === "dir" ? Math.round(size / 180_000) : 0,
      unreadable: false,
      mount: false,
    };
  });
  if (path === "/") {
    entries.push(
      {
        name: "boot",
        path: "/boot",
        kind: "dir",
        size: 0,
        files: 0,
        unreadable: false,
        mount: true,
      },
      {
        name: "root",
        path: "/root",
        kind: "dir",
        size: 4096,
        files: 0,
        unreadable: true,
        mount: false,
      },
    );
  }
  entries.sort((a, b) => b.size - a.size);
  return {
    path,
    total: entries.reduce((n, e) => n + e.size, 0) + 4096,
    entries,
    hidden_count: depth === 1 ? 312 : 0,
    hidden_size: depth === 1 ? Math.round(total * 0.01) : 0,
    unreadable: path === "/",
    incomplete: false,
    took_ms: 40,
  };
};

const osSummary = (): OsSummary => ({
  name: "Ubuntu 26.04.1 LTS",
  version: "26.04.1 LTS (Resolute Raccoon)",
  kernel: "7.0.0-34-generic",
  hostname: "talha-MS-7E02",
  architecture: "x86_64",
  boot_time: Math.floor(Date.now() / 1000) - 3 * 3600 - 14 * 60,
  details: [
    row("Operating system", "Name", "Ubuntu 26.04.1 LTS"),
    row("Operating system", "Version", "26.04.1 LTS (Resolute Raccoon)"),
    row("Operating system", "Codename", "resolute"),
    row("Operating system", "Distribution", "ubuntu"),
    row("Operating system", "Based on", "debian"),
    row("Operating system", "Website", "https://www.ubuntu.com/"),
    row("Operating system", "Host name", "talha-MS-7E02"),
    row("Operating system", "Architecture", "x86_64"),
    row("Kernel", "Release", "7.0.0-34-generic"),
    row("Kernel", "Build", "#34-Ubuntu SMP PREEMPT_DYNAMIC Wed Sep  2 14:29:37 UTC 2026"),
    row(
      "Kernel",
      "Command line",
      "BOOT_IMAGE=/boot/vmlinuz-7.0.0-34-generic root=UUID=c509a2d6 ro quiet splash",
    ),
    row("Kernel", "Loaded modules", "139"),
    row("Kernel", "Tainted", "Yes (flags 4097)"),
    row("Boot", "Firmware", "UEFI"),
    row("Boot", "Secure Boot", "Enabled"),
    row("Boot", "Started", "2026-10-04 05:36:06"),
    row(
      "Boot",
      "Startup took",
      "7.8s (firmware) + 2.7s (loader) + 2.5s (kernel) + 3.6s (initrd) + 9s (userspace) = 25.7s",
    ),
    row("Boot", "Init system", "systemd"),
    row("Boot", "Default target", "graphical.target"),
    row("Firmware", "Vendor", "American Megatrends International, LLC."),
    row("Firmware", "Version", "A.A0 (revision 5.27)"),
    row("Firmware", "Date", "09/27/2024"),
    row("Firmware", "Interface", "UEFI, 64-bit"),
    row(
      "Boot chain",
      "1 · Firmware",
      "UEFI — wakes the hardware and finds the bootloader · took 7.8s",
    ),
    row(
      "Boot chain",
      "2 · Bootloader",
      "GRUB — loads the kernel and the initramfs into memory · took 2.7s",
    ),
    row(
      "Boot chain",
      "3 · Kernel",
      "vmlinuz-7.0.0-34-generic (16.5 MiB) — takes over the machine · took 2.5s",
    ),
    row(
      "Boot chain",
      "4 · initramfs",
      "initrd.img-7.0.0-34-generic (37.4 MiB) — a temporary root filesystem whose drivers mount the real root disk · took 3.6s",
    ),
    row(
      "Boot chain",
      "5 · Init",
      "systemd — the first program; it starts everything else · took 8.5s",
    ),
    row("Session", "Current user", "talha"),
    row("Session", "Desktop", "ubuntu:GNOME"),
    row("Session", "Session type", "wayland"),
    row("Session", "Shell", "/bin/bash"),
    row("Language and time", "Language", "en_US.UTF-8"),
    row("Language and time", "Time zone", "Asia/Karachi"),
    row("Language and time", "Clock synchronised", "Yes"),
    row("Security", "AppArmor", "Enabled"),
    row("Security", "Kernel lockdown", "integrity"),
    row("Security", "Address randomisation", "Full (2)"),
    row("Virtualization", "Runs on", "Physical machine"),
    row("Software", "Package manager", "apt (dpkg)"),
    row("Software", "Installed packages", "2185"),
    row("Software", "Snap packages", "16"),
    row("Software", "C library", "ldd (Ubuntu GLIBC 2.43-2ubuntu2.4) 2.43"),
    row("Software", "systemd", "systemd 259 (259.5-0ubuntu3.4)"),
  ],
});

const moduleDefs: [string, number, string[]][] = [
  ["nvidia_modeset", 1572864, ["nvidia_drm"]],
  ["nvidia", 62914560, ["nvidia_modeset"]],
  ["btrfs", 2097152, []],
  ["snd_hda_intel", 61440, []],
  ["snd_hda_codec", 217088, ["snd_hda_intel", "snd_hda_codec_realtek"]],
  ["i915", 4194304, []],
  ["bluetooth", 1048576, ["btusb", "bnep"]],
  ["usbhid", 65536, []],
  ["ext4", 1146880, []],
  ["nf_tables", 372736, []],
];
const kernelModules = (): KernelModule[] =>
  moduleDefs
    .map(([name, size, used_by]) => ({ name, size, used_by }))
    .sort((a, b) => b.size - a.size);
const kernelModuleInfo = (name: string): ModuleInfo => {
  const m = kernelModules().find((x) => x.name === name);
  if (!m) return { name, found: false, details: [] };
  return {
    name,
    found: true,
    details: [
      row("Module", "Name", name),
      row("Module", "Description", `The ${name} driver`),
      row("Module", "License", "GPL"),
      row("Module", "File", `/lib/modules/7.0.0-34-generic/kernel/${name}.ko.zst`),
      row("Loaded", "Memory", `${(m.size / 1024).toFixed(0)} KiB`),
      row("Loaded", "Used by", m.used_by.join(", ") || "Nothing else"),
      row("Settings", "debug", "0 · Print extra messages (int)"),
    ],
  };
};

const GiB = 2 ** 30;

const osMemoryMock = (): OsMemory => ({
  total: 32 * GiB,
  used: 9.6 * GiB,
  available: 22.4 * GiB,
  swap_total: 15 * GiB,
  swap_used: 0,
  details: [
    row("Memory", "Total", "31.3 GiB"),
    row("Memory", "In use", "9.6 GiB"),
    row("Memory", "Available", "22.4 GiB"),
    row("Memory", "Disk cache", "19.3 GiB"),
    row("Memory", "Shared", "1.1 GiB"),
    row("Swap", "/dev/sdc1", "14.9 GiB partition · 0 B used · priority -2"),
    row("Swap", "Swappiness", "60 of 200: how eagerly RAM is swapped out"),
    row("Kernel's own use", "Slab", "812.4 MiB"),
    row("Kernel's own use", "Page tables", "61.2 MiB"),
    row("Kernel's own use", "Waiting to be written", "1.3 MiB"),
    row("Huge pages", "Transparent huge pages", "madvise"),
    row("Settings", "Overcommit", "Heuristic (0): large askings are refused"),
    row("Settings", "Memory map limit", "1048576"),
  ],
});

const osCgroupsMock = (): Cgroups => ({
  version: "v2 (unified)",
  controllers: ["cpuset", "cpu", "io", "memory", "hugetlb", "pids", "rdma", "misc"],
  groups: 142,
  capped: false,
  top: [
    { name: "system.slice", pids: 94, memory: 2.1 * GiB, groups: 71 },
    { name: "user.slice", pids: 61, memory: 6.4 * GiB, groups: 58 },
    { name: "machine.slice", pids: 8, memory: 0.6 * GiB, groups: 9 },
    { name: "init.scope", pids: 1, memory: 14 * 2 ** 20, groups: 0 },
  ],
});

const filesystemList = (): Filesystems => {
  const fs = (
    mount: string,
    source: string,
    fstype: string,
    size: number | null,
    used: number | null,
  ): Filesystems["rows"][number] => ({
    mount,
    source,
    fstype,
    size,
    used,
    available: size === null || used === null ? null : size - used,
    pseudo: size === null,
  });
  const rows = [
    fs("/mnt/archive", "/dev/sdb2", "ntfs", 2000 * GB, 1210 * GB),
    fs("/", "/dev/mapper/cryptroot", "btrfs", 970 * GB, 217 * GB),
    fs("/media/talha/STICK", "/dev/sdc1", "exfat", 62 * GB, 11 * GB),
    fs("/boot/efi", "/dev/nvme0n1p1", "vfat", 1.1 * GB, 0.006 * GB),
    fs("/dev/shm", "tmpfs", "tmpfs", 16 * GB, 0.1 * GB),
    fs("/run", "tmpfs", "tmpfs", 6.5 * GB, 0.003 * GB),
    fs("/proc", "proc", "proc", null, null),
    fs("/sys", "sysfs", "sysfs", null, null),
    fs("/sys/fs/cgroup", "cgroup2", "cgroup2", null, null),
  ];
  return { mounted: rows.length, real: rows.filter((r) => !r.pseudo).length, rows };
};

const osNetworkMock = (): OsNetwork => ({
  up: 1,
  total: 2,
  default_route: "via 192.168.1.1 on enp3s0",
  dns: ["192.168.1.1", "1.1.1.1"],
  listening_tcp: [22, 631, 1420, 5355],
  established: 14,
  interfaces: [
    {
      name: "enp3s0",
      kind: "ethernet",
      state: "UP",
      mac: "34:5a:60:57:b4:90",
      mtu: 1500,
      speed: "1000 Mb/s",
      ipv4: ["192.168.1.23/24"],
      ipv6: 2,
      rx: 18.4e9,
      tx: 1.9e9,
    },
    {
      name: "wlp4s0",
      kind: "wifi",
      state: "DOWN",
      mac: "a8:7e:ea:11:22:33",
      mtu: 1500,
      speed: null,
      ipv4: [],
      ipv6: 0,
      rx: 0,
      tx: 0,
    },
    {
      name: "lo",
      kind: "loopback",
      state: "UNKNOWN",
      mac: null,
      mtu: 65536,
      speed: null,
      ipv4: ["127.0.0.1/8"],
      ipv6: 1,
      rx: 42e6,
      tx: 42e6,
    },
  ],
  details: [
    row("Identity", "Host name", "talha-MS-7E02"),
    row("Reaching the internet", "Default route", "via 192.168.1.1 on enp3s0"),
    row("Reaching the internet", "DNS", "192.168.1.1, 1.1.1.1 · through systemd-resolved"),
    row("Open ports", "TCP, listening", "22, 631, 1420, 5355"),
    row("Open ports", "Established connections", "14"),
    row("Open ports", "UDP sockets", "9"),
  ],
});

const osSecurityMock = (): OsSecurity => ({
  apparmor: "Enabled",
  selinux: null,
  lockdown: "integrity",
  secure_boot: "Enabled",
  details: [
    row("Access control", "AppArmor", "Enabled"),
    row("Access control", "Program tracing", "Only parents and chosen debuggers (1)"),
    row("Kernel", "Lockdown", "integrity"),
    row("Kernel", "Address randomisation", "Full (2)"),
    row("Kernel", "Kernel addresses", "Hidden from ordinary users (1)"),
    row("Kernel", "Kernel log", "Administrators only"),
    row("Kernel", "Unprivileged BPF", "Blocked until restart"),
    row("Kernel", "User namespaces", "Ordinary users may create them"),
    row("Boot", "Secure Boot", "Enabled"),
    row("Network", "Firewall", "ufw is running"),
  ],
});

const pkgDefs: [string, string, string][] = [
  ["bash", "5.2.21-2ubuntu4", "amd64"],
  ["coreutils", "9.4-3ubuntu6", "amd64"],
  ["firefox", "1:1snap1-0ubuntu5", "amd64"],
  ["gnome-shell", "46.0-0ubuntu6", "amd64"],
  ["libc6", "2.39-0ubuntu8", "amd64"],
  ["libc6", "2.39-0ubuntu8", "i386"],
  ["linux-image-7.0.0-34-generic", "7.0.0-34.34", "amd64"],
  ["openssh-server", "1:9.6p1-3ubuntu13", "amd64"],
  ["python3", "3.12.3-0ubuntu2", "amd64"],
  ["systemd", "259.5-0ubuntu3.4", "amd64"],
  ["vim", "2:9.1.0016-1ubuntu7", "amd64"],
];
const osPackages = (): Packages => ({
  manager: "apt (dpkg)",
  packages: pkgDefs.map(([name, version, arch]): PackageRow => ({ name, version, arch })),
});

const osEnvironment = (): EnvVar[] =>
  [
    ["DISPLAY", ":0", false],
    ["HOME", "/home/talha", false],
    ["LANG", "en_US.UTF-8", false],
    ["PATH", "/usr/local/bin:/usr/bin:/bin:/home/talha/.cargo/bin", false],
    ["SHELL", "/bin/bash", false],
    ["SSH_AUTH_SOCK", "", true],
    ["USER", "talha", false],
    ["XDG_CURRENT_DESKTOP", "ubuntu:GNOME", false],
    ["XDG_SESSION_TYPE", "wayland", false],
    ["GITHUB_TOKEN", "", true],
  ]
    .map(([key, value, hidden]) => ({
      key: key as string,
      value: value as string,
      hidden: hidden as boolean,
    }))
    .sort((a, b) => a.key.localeCompare(b.key));

const serviceDefs: [string, string, string, string, string, number | null][] = [
  ["NetworkManager", "Network Manager", "active", "running", "enabled", 9.7e6],
  ["ssh", "OpenBSD Secure Shell server", "active", "running", "enabled", 5.1e6],
  ["cups", "CUPS Scheduler", "active", "running", "enabled", 12.4e6],
  ["bluetooth", "Bluetooth service", "active", "running", "enabled", 3.2e6],
  ["docker", "Docker Application Container Engine", "active", "running", "enabled", 148e6],
  ["gdm", "GNOME Display Manager", "active", "running", "enabled", 61e6],
  ["cron", "Regular background program processing daemon", "active", "running", "enabled", 0.9e6],
  ["alsa-restore", "Save/Restore Sound Card State", "active", "exited", "static", null],
  ["apt-daily", "Daily apt download activities", "inactive", "dead", "static", null],
  ["postgresql", "PostgreSQL RDBMS", "inactive", "dead", "disabled", null],
  ["nginx", "A high performance web server", "failed", "failed", "enabled", null],
  ["snapd", "Snap Daemon", "active", "running", "enabled", 28e6],
  ["ufw", "Uncomplicated firewall", "active", "exited", "enabled", null],
  ["avahi-daemon", "Avahi mDNS/DNS-SD Stack", "inactive", "dead", "masked", null],
];

const serviceList = (): ServiceSnapshot => {
  const services = serviceDefs.map(([name, description, active, sub, enabled, memory], i) => ({
    unit: `${name}.service`,
    name,
    description,
    load: "loaded",
    active,
    sub,
    enabled,
    main_pid: sub === "running" ? 700 + i * 113 : null,
    memory,
  }));
  return {
    available: true,
    overview: {
      total: services.length,
      running: services.filter((s) => s.sub === "running").length,
      exited: services.filter((s) => s.sub === "exited").length,
      failed: services.filter((s) => s.active === "failed").length,
      inactive: services.filter((s) => s.active === "inactive").length,
      enabled: services.filter((s) => s.enabled === "enabled").length,
      memory: services.reduce((n, s) => n + (s.memory ?? 0), 0),
    },
    services,
  };
};

const serviceDetail = (unit: string): ServiceDetail => {
  const s = serviceList().services.find((x) => x.unit === unit);
  if (!s) return { unit, found: false, details: [], logs: [], logs_note: null };
  return {
    unit,
    found: true,
    details: [
      row("Service", "Description", s.description),
      row("Service", "Loaded", s.load),
      row("Service", "Unit file", `/usr/lib/systemd/system/${unit}`),
      row("Service", "Starts at boot", s.enabled),
      row("State", "Status", `${s.active} (${s.sub})`),
      ...(s.active === "active"
        ? [row("State", "Active since", "Sat 2026-10-03 22:14:30 PKT")]
        : []),
      ...(s.active === "failed"
        ? [row("State", "Result", "exit-code"), row("State", "Exit status", "1")]
        : []),
      ...(s.main_pid ? [row("Process", "Main PID", String(s.main_pid))] : []),
      ...(s.memory ? [row("Process", "Memory", `${(s.memory / 1e6).toFixed(1)} MiB`)] : []),
      row("Command", "Command line", `/usr/sbin/${s.name} -D`),
      row("Run as", "User", "root"),
      row("Run as", "Restart policy", "on-failure"),
      row("Dependencies", "Starts after", "network.target, system.slice"),
      row("Dependencies", "Wanted by", "multi-user.target"),
    ],
    logs:
      s.active === "failed"
        ? [
            `2026-10-03T22:14:31+05:00 host ${s.name}[812]: bind() to 0.0.0.0:80 failed (98: Address already in use)`,
            `2026-10-03T22:14:31+05:00 host systemd[1]: ${unit}: Main process exited, code=exited, status=1/FAILURE`,
            `2026-10-03T22:14:31+05:00 host systemd[1]: ${unit}: Failed with result 'exit-code'.`,
          ]
        : [
            `2026-10-03T22:14:30+05:00 host systemd[1]: Started ${s.description}.`,
            `2026-10-03T22:14:31+05:00 host ${s.name}[${s.main_pid ?? 1}]: ready`,
          ],
    logs_note: null,
  };
};

const wave = (tick: number, base: number, amp: number, phase = 0) =>
  Math.max(0, base + Math.sin(tick / 3 + phase) * amp + Math.sin(tick * 1.7 + phase) * amp * 0.3);

const sample = (tick: number): Sample => ({
  t: Date.now(),
  cpu: {
    total: wave(tick, 24, 14),
    cores: Array.from({ length: 20 }, (_, i) =>
      Math.min(100, wave(tick, 20 + (i % 5) * 10, 22, i)),
    ),
    freq_mhz: 3200 + wave(tick, 0, 900),
    temperature: 55 + wave(tick, 0, 6),
    load: [1.43, 1.23, 1.22],
  },
  memory: {
    total: 32947e6,
    used: 9400e6 + wave(tick, 0, 400e6),
    available: 21500e6,
    cached: 14800e6,
    swap_total: 16000e6,
    swap_used: 2000e6 + tick * 4e6,
  },
  disks: [
    { name: "sda", model: "ST1000LM035", read_bps: 0, write_bps: 0, busy: 0 },
    {
      name: "sdc",
      model: "Samsung SSD 860",
      read_bps: wave(tick, 30e6, 25e6),
      write_bps: wave(tick, 8e6, 6e6, 2),
      busy: wave(tick, 35, 30),
    },
  ],
  volumes: [
    { mount: "/", file_system: "ext4", total: 474e9, used: 236e9 },
    { mount: "/boot/efi", file_system: "vfat", total: 1.1e9, used: 6.6e6 },
  ],
  gpus: [
    {
      id: "0000:01:00.0",
      vendor: "nvidia",
      kind: "discrete",
      name: "NVIDIA GeForce GTX 1060 3GB",
      util: wave(tick, 30, 25),
      memory_used: 875e6,
      memory_total: 3221e6,
      temperature: 52 + wave(tick, 0, 4),
      power: 15 + wave(tick, 0, 10),
      power_limit: 120,
      core_mhz: 1582,
      memory_mhz: 4006,
      fan: 48,
    },
    {
      id: "0000:00:02.0",
      vendor: "intel",
      kind: "integrated",
      name: "Intel UHD Graphics 630",
      util: null,
      memory_used: null,
      memory_total: null,
      temperature: null,
      power: null,
      power_limit: null,
      core_mhz: 350 + wave(tick, 0, 300),
      memory_mhz: null,
      fan: null,
    },
  ],
  net: [
    {
      name: "enp3s0",
      kind: "ethernet",
      rx_bps: wave(tick, 2.2e6, 2e6),
      tx_bps: wave(tick, 0.3e6, 0.25e6, 1),
      rx_total: 6.6e9,
      tx_total: 4.1e8,
      speed_mbps: 1000,
      default_route: true,
    },
  ],
  thermal: {
    sensors: [
      {
        id: "hwmon0:temp1",
        group: "Motherboard",
        label: "Motherboard",
        celsius: 28 + wave(tick, 0, 0.6),
        high: null,
        critical: null,
      },
      {
        id: "hwmon1:temp1",
        group: "CPU",
        label: "Package id 0",
        celsius: 62 + wave(tick, 0, 7),
        high: 80,
        critical: 100,
      },
      ...[0, 4, 8, 12, 16, 20].map((c, i) => ({
        id: `hwmon1:temp${2 + i * 4}`,
        group: "CPU",
        label: `Core ${c}`,
        celsius: 54 + wave(tick, 0, 8, i),
        high: 80,
        critical: 100,
      })),
      {
        id: "hwmon2:temp1",
        group: "NVMe · Samsung 970 EVO",
        label: "Composite",
        celsius: 41 + wave(tick, 0, 3, 1),
        high: 75,
        critical: 85,
      },
      {
        id: "hwmon2:temp2",
        group: "NVMe · Samsung 970 EVO",
        label: "Sensor 1",
        celsius: 44 + wave(tick, 0, 3, 2),
        high: 75,
        critical: 85,
      },
      {
        id: "hwmon3:temp1",
        group: "Wi-Fi adapter",
        label: "Wi-Fi adapter",
        celsius: 36 + wave(tick, 0, 1.5, 3),
        high: null,
        critical: null,
      },
    ],
    fans: [
      { id: "hwmon4:fan1", group: "Motherboard", label: "CPU fan", rpm: 1180 + wave(tick, 0, 140) },
      {
        id: "hwmon4:fan2",
        group: "Motherboard",
        label: "Case fan",
        rpm: 760 + wave(tick, 0, 60, 1),
      },
    ],
  },
  power: {
    cpu_watts: null,
    cpu_readable: false,
    cpu_limit_sustained: 135,
    cpu_limit_boost: 150,
    ac_online: false,
    batteries: [
      {
        name: "BAT0",
        percent: Math.max(5, 78 - tick * 0.05),
        status: "Discharging",
        watts: 11 + wave(tick, 0, 3),
        seconds_left: 14400 - tick * 20,
        health: 91,
        cycles: 214,
      },
    ],
  },
});

export const fixtures = {
  hardware,
  connections,
  components,
  memoryModules,
  processList,
  processDetail,
  osSummary,
  kernelModules,
  kernelModuleInfo,
  osPackages,
  osEnvironment,
  osMemoryMock,
  osSecurityMock,
  osCgroupsMock,
  osNetworkMock,
  filesystemList,
  diskDevices,
  directoryUsage,
  serviceList,
  serviceDetail,
  accountList,
  namespaceList,
  sample,
};

let installed = false;

export function installMockBackend(): void {
  if (installed) return;
  installed = true;

  const params = new URLSearchParams(globalThis.location?.search ?? "");
  const net = params.get("net") ?? "wired";
  let tick = 0;

  mockIPC((command, payload) => {
    const args = (payload ?? {}) as Record<string, unknown>;
    switch (command) {
      case "hardware_info":
        return hardware(net);
      case "system_info":
        return { computer_name: "talha-MS-7E02", components } satisfies SystemInfo;
      case "read_memory_modules":
        return new Promise((resolve) => setTimeout(() => resolve(memoryModules), 400));
      case "device_report":
        return [row("Technical", "Deep row", `from ${String(args.id)}`)];
      case "process_list":
        return processList(++tick);
      case "process_detail":
        return processDetail(Number(args.pid));
      case "account_list":
        return accountList();
      case "namespace_list":
        return namespaceList();
      case "os_summary":
        return osSummary();
      case "kernel_modules":
        return kernelModules();
      case "kernel_module_info":
        return kernelModuleInfo(String(args.name));
      case "os_packages":
        return osPackages();
      case "os_environment":
        return osEnvironment();
      case "os_memory":
        return osMemoryMock();
      case "os_security":
        return osSecurityMock();
      case "os_cgroups":
        return osCgroupsMock();
      case "os_network":
        return osNetworkMock();
      case "filesystem_list":
        return filesystemList();
      case "disk_devices":
        return diskDevices();
      case "directory_usage":
        return new Promise((resolve) =>
          setTimeout(() => resolve(directoryUsage(String(args.path))), 250),
        );
      case "disk_mount":
        return new Promise((resolve) =>
          setTimeout(() => {
            const at = `/media/talha/${String(args.device).split("/").pop()}`;
            mountedHere.set(String(args.device), at);
            resolve(at);
          }, 500),
        );
      case "path_open":
      case "process_signal":
      case "service_action":
        return null;
      case "service_list":
        return serviceList();
      case "service_detail":
        return serviceDetail(String(args.unit));
      case "monitor_sample":
        return sample(++tick);
      default:
        throw new Error(`mock backend: unknown command "${command}"`);
    }
  });
}
