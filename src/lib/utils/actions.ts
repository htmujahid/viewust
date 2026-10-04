import { errorMessage } from "$lib/api/client";
import { confirmation, type ConfirmOptions } from "$lib/stores/confirm.svelte";
import { toasts } from "$lib/stores/toast.svelte";

export async function copyText(text: string, what: string): Promise<void> {
  try {
    await navigator.clipboard.writeText(text);
    toasts.show(`Copied ${what}`, "ok");
  } catch {
    toasts.show(`Couldn't copy ${what}`, "danger");
  }
}

/**
 * Runs a backend action and tells the person how it went. `ask` puts a confirmation in front of it;
 * `after` runs on success (usually a refresh of the list the action changed).
 */
export async function perform(
  done: string,
  run: () => Promise<unknown>,
  options: { ask?: ConfirmOptions; after?: () => void | Promise<void> } = {},
): Promise<void> {
  if (options.ask && !(await confirmation.ask(options.ask))) return;
  try {
    await run();
    toasts.show(done, "ok");
    await options.after?.();
  } catch (e) {
    toasts.show(errorMessage(e), "danger");
  }
}
