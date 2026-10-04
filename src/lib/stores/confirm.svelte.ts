export interface ConfirmOptions {
  title: string;
  body?: string;
  confirmLabel?: string;
  danger?: boolean;
}

class ConfirmStore {
  options = $state.raw<ConfirmOptions | null>(null);
  #resolve: ((ok: boolean) => void) | null = null;

  /** Resolves true when the person confirms, false when they cancel or look away. */
  ask(options: ConfirmOptions): Promise<boolean> {
    this.#resolve?.(false);
    this.options = options;
    return new Promise((resolve) => (this.#resolve = resolve));
  }

  answer(ok: boolean) {
    this.#resolve?.(ok);
    this.#resolve = null;
    this.options = null;
  }
}

export const confirmation = new ConfirmStore();
