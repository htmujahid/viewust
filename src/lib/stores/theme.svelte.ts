export type ThemeMode = "system" | "light" | "dark";

function load(): ThemeMode {
  try {
    const saved = localStorage.getItem("theme");
    if (saved === "light" || saved === "dark") return saved;
  } catch {}
  return "system";
}

class Theme {
  mode = $state<ThemeMode>("system");

  init() {
    this.mode = load();
  }

  /** What the screen actually shows right now, with "system" resolved. */
  get effective(): "light" | "dark" {
    if (this.mode !== "system") return this.mode;
    try {
      return window.matchMedia("(prefers-color-scheme: dark)").matches ? "dark" : "light";
    } catch {
      return "dark";
    }
  }

  /** Flips to the opposite of whatever is on screen. */
  toggle() {
    this.set(this.effective === "dark" ? "light" : "dark");
  }

  set(mode: ThemeMode) {
    this.mode = mode;
    const root = document.documentElement;
    if (mode === "system") delete root.dataset.theme;
    else root.dataset.theme = mode;
    try {
      if (mode === "system") localStorage.removeItem("theme");
      else localStorage.setItem("theme", mode);
    } catch {}
  }
}

export const theme = new Theme();
