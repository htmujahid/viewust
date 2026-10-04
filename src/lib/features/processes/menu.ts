import { goto } from "$app/navigation";

import { api } from "$lib/api/client";
import type { ProcessRow, Signal } from "$lib/api/types";
import { accounts } from "$lib/features/accounts/store.svelte";
import { separator, type MenuItem } from "$lib/stores/menu.svelte";
import { copyText, perform } from "$lib/utils/actions";

import { processes } from "./store.svelte";

const SENT: Record<Signal, string> = {
  term: "Asked {name} to quit",
  kill: "Force-killed {name}",
  stop: "Paused {name}",
  cont: "Resumed {name}",
};

export function signalProcess(p: ProcessRow, signal: Signal): Promise<void> {
  const ask =
    signal === "term"
      ? {
          title: `Quit ${p.name}?`,
          body: `PID ${p.pid} will be asked to close. Unsaved work in it may be lost.`,
          confirmLabel: "Quit",
          danger: true,
        }
      : signal === "kill"
        ? {
            title: `Force-kill ${p.name}?`,
            body: `PID ${p.pid} will be ended immediately and gets no chance to save anything. Use this only when asking it to quit doesn't work.`,
            confirmLabel: "Force kill",
            danger: true,
          }
        : undefined;
  return perform(
    SENT[signal].replace("{name}", `${p.name} (${p.pid})`),
    () => api.processSignal(p.pid, signal),
    { ask, after: processes.refresh },
  );
}

export function processMenu(p: ProcessRow): MenuItem[] {
  const guarded = p.pid <= 1 || p.kernel;
  const paused = p.state === "Stopped";
  return [
    { label: "View details", onselect: () => processes.select(p.pid) },
    { label: "Copy PID", hint: String(p.pid), onselect: () => copyText(String(p.pid), "PID") },
    { label: "Copy name", onselect: () => copyText(p.name, "name") },
    separator,
    {
      label: `Show ${p.user}'s account`,
      onselect: () => {
        accounts.openUser(p.user);
        return goto("/accounts");
      },
    },
    {
      label: `Only ${p.user}'s processes`,
      onselect: () => {
        processes.search = p.user;
        processes.select(null);
      },
    },
    separator,
    paused
      ? {
          label: "Resume",
          hint: "SIGCONT",
          disabled: guarded,
          onselect: () => signalProcess(p, "cont"),
        }
      : {
          label: "Pause",
          hint: "SIGSTOP",
          disabled: guarded,
          onselect: () => signalProcess(p, "stop"),
        },
    {
      label: "Quit",
      hint: "SIGTERM",
      danger: true,
      disabled: guarded,
      onselect: () => signalProcess(p, "term"),
    },
    {
      label: "Force kill",
      hint: "SIGKILL",
      danger: true,
      disabled: guarded,
      onselect: () => signalProcess(p, "kill"),
    },
  ];
}
