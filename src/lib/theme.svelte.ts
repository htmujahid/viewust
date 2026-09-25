export type ThemeMode = "system" | "light" | "dark";

function load(): ThemeMode {
  try {
    const saved = localStorage.getItem("theme");
    if (saved === "light" || saved === "dark") return saved;
  } catch {
    // storage unavailable: fall back to following the system
  }
  return "system";
}

class Theme {
  mode = $state<ThemeMode>("system");

  init() {
    this.mode = load();
  }

  set(mode: ThemeMode) {
    this.mode = mode;
    const root = document.documentElement;
    if (mode === "system") delete root.dataset.theme;
    else root.dataset.theme = mode;
    try {
      if (mode === "system") localStorage.removeItem("theme");
      else localStorage.setItem("theme", mode);
    } catch {
      // not persisted; still applies for this session
    }
  }
}

export const theme = new Theme();
