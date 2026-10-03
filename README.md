# Viewust

A live map of everything connected to your computer, and a window into it.

- **Overview** – the home page (click the logo): the computer's identity and health at a glance, live
  processor, memory, graphics and disk tiles, and a summary of every part and connected device.
- **Devices** – every external device (keyboards, mice, webcams, monitors, drives…) drawn around
  your computer, wired or wireless, plus how the computer reaches the internet.
- **Inside the computer** – motherboard, processor, memory, drives, graphics card, power.
- **Processes** – every running program and how it uses memory, plus (on Linux) the namespaces that
  isolate programs from one another.
- **Live monitor** – real-time processor, memory, storage, graphics, thermal, power and network charts.
- **Accounts** – users and groups: who is on the computer and what they belong to.
- **Services** – every systemd service, its state, memory and recent log.

Click anything for a summary, double-click for its full technical page.

Built with [Tauri 2](https://tauri.app) (Rust) and [SvelteKit](https://svelte.dev) (Svelte 5, TypeScript).
Hardware reading is **Linux-first**; other systems get the basics.

## Running it

| Command          | What it does                                                                                                                                                                                       |
| ---------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `pnpm tauri dev` | Run the real app (Rust backend + window).                                                                                                                                                          |
| `pnpm dev:mock`  | Run the interface in a browser with sample data. No desktop shell needed. Add `?net=wifi` or `?net=offline` to try the other connection states, or `?os=windows` to see pages that are Linux-only. |
| `pnpm verify`    | Everything CI would check: formatting, type check, unit tests, build, and the Rust format/lint/tests.                                                                                              |

Other scripts: `pnpm test`, `pnpm format`, `pnpm check`, `pnpm rust:verify`.

Needs [Rust](https://rustup.rs), [pnpm](https://pnpm.io) and Tauri's
[Linux prerequisites](https://tauri.app/start/prerequisites/). A few readings use optional system tools
(`nvidia-smi`, `amixer`, `modinfo`, `nmcli`, `iw`); the app works without them and simply shows less.

## How it's organised

See **[docs/ARCHITECTURE.md](docs/ARCHITECTURE.md)** for the layout, the conventions, and
step-by-step recipes for adding a command, a page or a new kind of device.
