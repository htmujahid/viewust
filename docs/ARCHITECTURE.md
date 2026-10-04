# Architecture

Two halves that only meet at one narrow seam:

```
 SvelteKit UI  ──  src/lib/api/client.ts  ──  Tauri commands  ──  Rust modules
 (src/)             one typed wrapper          (async, thin)       (src-tauri/src/)
```

## Rust (`src-tauri/src/`)

One module per feature. Each has the same shape:

```
feature/
  mod.rs        what the module exposes
  model.rs      the serde types sent to the UI (their shape is a contract)
  commands.rs   the Tauri layer: thin, async, no logic
  …             the logic, split by concern
```

| Module       | Responsibility                                                                                      |
| ------------ | --------------------------------------------------------------------------------------------------- |
| `hardware`   | external devices: USB (`usb/`), audio jacks, monitors (`edid.rs`)                                   |
| `connection` | the route to the internet: wired/Wi-Fi, gateway, DNS, signal                                        |
| `system`     | internal parts: board, CPU, memory, drives, GPU, network, sound, power                              |
| `report`     | the deep technical report for one device (descriptors, HID, drivers…)                               |
| `disk_usage` | attached drives and partitions (`lsblk`, `df`), mounting (`udisksctl`), folder sizes for the tree   |
| `os`         | the OS by area: summary, cgroups, modules, memory, network, security, packages, environment…        |
| `processes`  | the process list and one process in depth                                                           |
| `monitor`    | one-second live samples                                                                             |
| `common`     | shared plumbing with no domain knowledge: details rows, formatting, sysfs, ID databases, `blocking` |
| `error`      | the one `AppError` every command returns; serialises to a plain message                             |

**Conventions**

- **Commands are `async` and never block.** Tauri runs a plain `fn` command on the main thread, so a
  slow read (`nvidia-smi`, a password prompt) would freeze the window. Commands hand their work to
  `common::blocking::run`, which uses `spawn_blocking`.
- **State is managed, not global.** `ProcessService` and `MonitorService` are registered with
  `.manage(...)` in `lib.rs` and injected with `State<'_, T>`. They hold what must survive between
  calls (CPU usage and rates are differences between two readings).
- **Reading never needs admin rights.** Changing things is explicit and narrow: the commands that do
  (`process_signal`, `service_action`, `disk_mount`) accept only a fixed set of actions,
  validate their target, refuse system-critical ones, and use `common::elevate` (`pkexec`) when root is
  needed. RAM modules (`system::memory::read_modules`) use the same helper.
- **Be honest about gaps.** If the OS doesn't report something (the power supply, a mouse's sensor),
  say so in the result rather than guessing.
- **Linux only, on purpose.** There are no `cfg` gates or per-OS fallbacks: modules read `/proc`,
  `/sys`, `/etc` and tools like `lsblk`, `df`, `systemctl` and `udevadm` directly, so each view can
  go as deep as the platform allows. Other operating systems will get their own codebase.
- **Long reads are cached and bounded.** The folder-size scan (`disk_usage::directory`) runs on a
  work queue across threads, stays on one device, never follows links, counts hard links once, and
  stops at a deadline (reporting `incomplete`). `UsageService` remembers results so reopening a
  folder is instant; a rescan passes `refresh`.
- **Parsers are pure functions with tests** (`parse_dmidecode`, `decode_hid`, `parse_edid`, …). Keep
  that split: reading a file is one function, interpreting its text is another.
- Lints: `unsafe_code = "forbid"`, clippy clean (`-D warnings` in `pnpm rust:verify`).

## Frontend (`src/`)

```
routes/            thin pages: compose components, own no styling or logic they could share
lib/
  api/             client.ts (the only place that calls invoke), types/ (per domain), mock.ts
  components/
    ui/            small generic pieces: Badge, Meter, DetailList, PageHeader, TabNav…
    charts/        LineChart (+ pure axis scale)
  map/             the diagram shell: MapView, nodes, sidebar, toolbar, legend
  illustrations/   Illustration.svelte, drawings/ (one file per drawing), sizes.ts, kinds.ts
  features/        one folder per area: devices, internals, device, processes, monitor
  stores/          app-wide state (theme)
  utils/           pure helpers: format, details grouping, Poller
styles/            tokens.css (colour/spacing/type), base.css, utilities.css
```

**Conventions**

- **Pages are thin.** `routes/+page.svelte` is ~40 lines: it picks data and passes it to `MapView`.
- **One place calls the backend.** Use `api.*` from `lib/api/client.ts`. It gives every command a
  return type, and a missing backend (a plain browser tab) fails with advice instead of a stack trace.
- **Stores are classes with `$state`**, one per feature (`features/*/store.svelte.ts`). Live ones use
  `Poller`, which never overlaps runs.
- **Logic that can be pure is pure and tested** (`graph.ts`, `series.ts`, `details.ts`, `scale.ts`).
  Components render; they don't compute layouts.
- **The sidebar is the three hardware pages plus the operating system.** `lib/nav.ts` maps paths to
  sidebar entries. The OS page is a dashboard: one card per subsystem with live figures
  (`features/os/subsystems.ts` builds them as pure, tested functions), each linking to its full
  page; those pages carry a back link instead of tabs.
- **Navigation is links.** Use `<a href>` (or `BackLink`/`NavLinks`); reserve `goto` for redirects and
  keyboard shortcuts.
- **Style with tokens.** Colours, spacing (`--s-*`), type (`--fs-*`) and radii live in
  `styles/tokens.css`. Light and dark are both defined there; never hard-code a colour in a component.
- **Chart colours** are `--series-1` (blue) then `--series-2` (orange): a validated, colour-blind-safe
  pair. Single-series charts use one hue.
- **Run without the desktop:** `pnpm dev:mock` installs `lib/api/mock.ts` (Tauri's `mockIPC`), so
  every page works in a browser. The same fixtures drive the unit tests.

## Recipes

### Add a backend command

1. In the feature's `model.rs`, add the response type (`#[derive(Serialize)]`).
2. Write the logic in the feature module; keep parsing pure and add a test.
3. In `commands.rs`: `#[tauri::command] pub async fn name(...) -> Result<T>` that calls
   `blocking::run(move || …).await`.
4. Register it in `lib.rs`'s `generate_handler![…]`.
5. Mirror the type in `src/lib/api/types/<domain>.ts` and add one line to `api` in `client.ts`.
6. Add a fixture case in `lib/api/mock.ts`.

### Add a right-click menu

1. Build the items in `features/<name>/menu.ts` as `MenuItem[]` (`label`, `onselect`, optional `hint`,
   `danger`, `disabled`; use `separator` between groups).
2. Run backend work through `perform(done, run, { ask, after })` from `utils/actions.ts`: it confirms
   first when `ask` is given, shows a toast with the result or the error, then refreshes.
3. Open it with `menu.open(event, items, title)`. `DataTable` takes `oncontext={(row, e) => …}`.

The menu, confirmation dialog and toasts are mounted once in `routes/+layout.svelte`.

### Add a page

Create `routes/<name>/+page.svelte`. Wrap it in `<SectionShell title=…>` (back link + title; pages
have no tab bars — every page is reached from the OS dashboard or the sidebar), put the logic in a
store under `lib/features/<name>/`, and add its card to the OS dashboard or `lib/nav.ts`.

### Add a new kind of device

1. `illustrations/kinds.ts`: add the kind. TypeScript will now point at everything that needs it.
2. `illustrations/drawings/<Kind>.svelte`: the SVG, using the shared palette classes
   (`body`, `dark`, `accent`, `faint`, `gold`, `led`…). Register it in `drawings/index.ts`.
3. `illustrations/sizes.ts`: its real-world size in cm (or a fixed `px` for internal parts).
4. `map/model.ts`: its label in `KIND_LABELS`.
5. Backend: teach `hardware/usb/classify.rs` to recognise it, and where it sits on the map in
   `features/devices/graph.ts` (`LEFT` / `RIGHT` zones; everything else goes below).

## Checks

`pnpm verify` runs: Prettier, `svelte-check` (with unused-code errors on), Vitest, the production
build, `cargo fmt --check`, `cargo clippy -D warnings` and `cargo test`.
