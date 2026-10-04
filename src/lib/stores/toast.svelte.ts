export type ToastTone = "ok" | "danger" | "neutral";

export interface Toast {
  id: number;
  text: string;
  tone: ToastTone;
}

class Toasts {
  items = $state.raw<Toast[]>([]);
  #next = 1;

  show(text: string, tone: ToastTone = "neutral", ms = tone === "danger" ? 7000 : 3500) {
    const id = this.#next++;
    this.items = [...this.items.slice(-3), { id, text, tone }];
    setTimeout(() => this.dismiss(id), ms);
  }

  dismiss(id: number) {
    this.items = this.items.filter((t) => t.id !== id);
  }
}

export const toasts = new Toasts();
