import { mockIPC } from "@tauri-apps/api/mocks";

import type {
  Component,
  Connection,
  Detail,
  FilesystemKind,
  FilesystemRow,
  FilesystemSnapshot,
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

const filesystemDefs: [
  string,
  string,
  string,
  FilesystemKind,
  number | null,
  number,
  { label?: string; ro?: boolean; note?: string }?,
][] = [
  [
    "/",
    "/dev/nvme0n1p2",
    "ext4",
    "disk",
    474 * GB,
    217 * GB,
    { note: "nvme0n1p2 (part) → nvme0n1 (disk)" },
  ],
  ["/boot/efi", "/dev/nvme0n1p1", "vfat", "disk", 1.1 * GB, 0.006 * GB],
  ["/home", "/dev/mapper/vg-home", "btrfs", "disk", 930 * GB, 812 * GB, { label: "home" }],
  ["/mnt/archive", "/dev/sda1", "xfs", "disk", 4000 * GB, 1210 * GB, { label: "archive" }],
  ["/media/talha/STICK", "/dev/sdb1", "exfat", "removable", 62 * GB, 11 * GB, { label: "STICK" }],
  [
    "/media/talha/Backup",
    "/dev/sdc1",
    "ntfs3",
    "removable",
    1000 * GB,
    968 * GB,
    { label: "Backup" },
  ],
  ["/run/media/talha/INSTALL", "/dev/sr0", "iso9660", "optical", 3.4 * GB, 3.4 * GB, { ro: true }],
  ["/mnt/nas", "nas.local:/volume1/media", "nfs4", "network", 7200 * GB, 5100 * GB],
  ["/mnt/work", "//fileserver/work", "cifs", "network", 2000 * GB, 1980 * GB],
  ["/dev/shm", "tmpfs", "tmpfs", "memory", 16 * GB, 0.1 * GB],
  ["/run", "tmpfs", "tmpfs", "memory", 6.5 * GB, 0.003 * GB],
  ["/tmp", "tmpfs", "tmpfs", "memory", 16 * GB, 0.8 * GB],
  ["/snap/core22/1722", "/dev/loop0", "squashfs", "image", 0.07 * GB, 0.07 * GB, { ro: true }],
  ["/snap/firefox/4650", "/dev/loop1", "squashfs", "image", 0.26 * GB, 0.26 * GB, { ro: true }],
  ["/var/lib/docker/overlay2/merged", "overlay", "overlay", "overlay", 474 * GB, 217 * GB],
  ["/proc", "proc", "proc", "virtual", null, 0],
  ["/sys", "sysfs", "sysfs", "virtual", null, 0],
  ["/sys/fs/cgroup", "cgroup2", "cgroup2", "virtual", null, 0],
  ["/dev", "devtmpfs", "devtmpfs", "memory", 16 * GB, 0],
];

const filesystemList = (): FilesystemSnapshot => {
  const filesystems: FilesystemRow[] = filesystemDefs.map(
    ([mount, source, fstype, kind, size, used, extra]) => {
      const read_only = extra?.ro ?? false;
      const available = size === null ? null : size - used;
      return {
        mount,
        source,
        fstype,
        kind,
        read_only,
        size,
        used: size === null ? null : used,
        available,
        inodes_total: size === null ? null : Math.round(size / 16384),
        inodes_used: size === null ? null : Math.round(used / 65536),
        label: extra?.label ?? null,
        details: [
          row("Filesystem", "Type", fstype),
          row("Filesystem", "Mounted at", mount),
          row("Filesystem", "Source", source),
          row("Filesystem", "Access", read_only ? "Read-only" : "Read and write"),
          ...(size === null
            ? []
            : [
                row("Space", "Size", `${(size / 2 ** 30).toFixed(1)} GiB`),
                row("Space", "Used", `${(used / 2 ** 30).toFixed(1)} GiB`),
                row("Space", "Available", `${((available ?? 0) / 2 ** 30).toFixed(1)} GiB`),
              ]),
          ...(extra?.note ? [row("Device", "Built on", extra.note)] : []),
          row("Options", "Mount options", read_only ? "ro,relatime" : "rw,relatime"),
        ],
      };
    },
  );
  const counted = filesystems.filter(
    (f) => ["disk", "removable", "network"].includes(f.kind) && f.size !== null,
  );
  return {
    overview: {
      size: counted.reduce((n, f) => n + (f.size ?? 0), 0),
      used: counted.reduce((n, f) => n + (f.used ?? 0), 0),
      available: counted.reduce((n, f) => n + (f.available ?? 0), 0),
      volumes: counted.length,
    },
    filesystems,
  };
};

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
  filesystemList,
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
      case "filesystem_list":
        return filesystemList();
      case "process_signal":
      case "service_action":
      case "filesystem_action":
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
