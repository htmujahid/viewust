# Viewust

A desktop app that shows everything connected to your computer, what's inside it, and what it's
doing right now.

## What it shows

- **Overview** – the computer at a glance: identity, health, live load, and a summary of every part.
- **Devices** – external devices (keyboards, mice, monitors, drives…) and the route to the internet.
- **System** – the internal parts: motherboard, processor, memory, drives, graphics, power.
- **Live monitor** – real-time processor, memory, storage, graphics, thermal, power and network charts.
- **Processes** – running programs and their memory use, plus Linux namespaces.
- **Disk usage** – every connected drive (internal, USB, optical, network shares) as a tree: drive →
  partitions (through encrypted or LVM layers) → folders, to see what is taking the space. Drives
  that aren't mounted are listed too, with a Mount button so you can browse them.
- **Services** – systemd services, their state and recent logs.
- **Accounts** – users and groups.
- **Operating system** – the distribution and version, kernel, boot and firmware (UEFI, Secure Boot,
  startup time), desktop session, language and time, security (AppArmor/SELinux, lockdown),
  virtualization and software, plus tabs for loaded kernel modules (with their settings), every
  installed package, and the session's environment variables (secrets are never loaded). Links to
  all the related pages are on its overview.

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
