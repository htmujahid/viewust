# Viewust

A desktop app that shows everything connected to your computer, what's inside it, and what it's
doing right now.

## What it shows

- **Overview** – the computer at a glance: identity, health, live load, and a summary of every part.
- **Devices** – external devices (keyboards, mice, monitors, drives…) and the route to the internet.
- **System** – the internal parts: motherboard, processor, memory, drives, graphics, power.
- **Live monitor** – real-time processor, memory, storage, graphics, thermal, power and network charts.
- **Processes** – running programs and their memory use, plus Linux namespaces.
- **Services** – systemd services, their state and recent logs.
- **Accounts** – users and groups.

Click anything for a summary; double-click a device or part for its full technical page.

## Screenshots

![Devices page](docs/screenshots/devices.png)

![System page](docs/screenshots/system.png)

_Sample data, not a real machine._

## Platforms

Built for **Linux** first. Users, groups, processes and the basics of the hardware views work on
other systems; namespaces, services and some hardware details are Linux-only and say so when
unavailable. Everything is read-only; the one exception is memory-module details, which ask for
permission first.

## Built with

[Tauri 2](https://tauri.app) (Rust) and [SvelteKit](https://svelte.dev) (Svelte 5, TypeScript).
