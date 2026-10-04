# Viewust

A desktop app that shows everything connected to your computer, what's inside it, and what it's
doing right now.

## What it shows

- **Overview** – the computer at a glance: identity, health, live load, and a summary of every part.
- **Health** – one verdict across the whole machine: failed services, journal errors, full disks,
  memory and pressure, temperatures against their limits, kernel warnings, a pending kernel
  restart, battery wear — each linking to where to look closer.

**Hardware** – the physical computer:

- **Devices** – external devices (keyboards, mice, monitors, drives…) and the route to the internet.
- **System** – the internal parts: motherboard, processor, memory, drives, graphics, power.
- **Live monitor** – real-time processor, memory, storage, graphics, thermal, power and network
  charts, plus a live audio view: what is playing or recording (and by which program), at what
  rate and format, every output and input with volume and mute, and the sound cards behind them.

**Operating system** – what it runs:

- **Overview** – one page, grouped the way an OS is structured: **System** (distribution, boot,
  session), **Kernel** (kernel & drivers, memory), **Processes** (programs, services, namespaces,
  control groups), **Storage** (disk usage, file systems), **Network**, **Accounts**, **Security**
  and **Software** — every card with live figures and a button to its page.
- **Kernel & drivers** – release, build and command line, plus every loaded module and its settings.
- **Memory management** – RAM, swap devices, caches, the kernel's own use, and memory settings.
- **Control groups** – cgroup version and controllers, and the top-level slices with their
  processes and memory.
- **Security & protection** – AppArmor/SELinux, kernel lockdown and hardening, Secure Boot, firewall.
- **Packages and environment** – every installed package, and the session's environment variables
  (secrets are never loaded).
- **Processes** – running programs and their memory use, plus Linux namespaces.
- **Services** – systemd services, their state and recent logs.
- **Disk usage** – every connected drive (internal, USB, optical, network shares) as a tree: drive →
  partitions (through encrypted or LVM layers) → folders, to see what is taking the space. Drives
  that aren't mounted are listed too, with a Mount button so you can browse them.
- **File systems** – everything mounted, from drives to the kernel's own views, with space and use.
- **Networking** – interfaces with addresses, MAC, link speed and traffic; the default route, DNS,
  listening TCP ports and established connections — plus a Connections page showing who this
  computer is talking to, address by address.
- **Logs** – the journal's size and boots kept, and every error-level message of this boot.
- **Logins** – the signed-in sessions: user, kind, where from (remote ones flagged), since when.
- **Virtualization** (top level) – the machine from the guests' side: whether the processor and
  kernel can host (VT-x/AMD-V, /dev/kvm, nested), whether this system is itself a guest, the
  docker/podman containers and libvirt/machined VMs, and what guests take in storage (overlay
  roots, runtime disk use), control groups, namespaces and virtual networks.
- **Accounts** – users and groups.

The sidebar carries the three hardware pages and the Operating system; processes, services, disk
usage and accounts are reached through the OS overview.

Click anything for a summary; double-click a device or part for its full technical page.

## Screenshots

![Devices page](docs/screenshots/devices.png)

![System page](docs/screenshots/system.png)

_Sample data, not a real machine._

## Platforms

**Linux only**, by design. Viewust reads `/proc`, `/sys` and the system's own tools directly, so it
can show far more than a cross-platform abstraction would. Windows and macOS versions would be
separate codebases.

## Right-click actions

Right-click anything in a table or on the map for a menu:

- **Processes** – view, copy, jump to the owner's account, pause/resume, quit, force kill.
- **Services** – view, jump to the main process, start, stop, restart, enable or disable at boot.
- **Disk usage** – expand or collapse, mount a drive, open in the file manager, copy the path,
  rescan a folder.
- **Accounts, groups, namespaces, devices** – view, copy, and jump to related pages.
- **Operating system** – view a kernel module, copy names, versions and values.

Viewing is read-only and needs no permission. Anything that changes the system asks first: stopping
a program or unmounting a drive shows a confirmation, and actions that need administrator rights
(service control, unmounting some shares) ask the desktop for your password. System-critical targets
(PID 1, `/`, `/boot`, …) are refused.

## Built with

[Tauri 2](https://tauri.app) (Rust) and [SvelteKit](https://svelte.dev) (Svelte 5, TypeScript).
