export type ThemeMode = "light" | "dark" | "system";

const MEDIA_QUERY = "(prefers-color-scheme: dark)";

let mediaQueryCleanup: (() => void) | null = null;

function getSystemTheme(): "light" | "dark" {
  if (typeof window === "undefined") return "light";
  return window.matchMedia(MEDIA_QUERY).matches ? "dark" : "light";
}

export function resolveTheme(mode: ThemeMode): "light" | "dark" {
  if (mode === "system") return getSystemTheme();
  return mode;
}

export function applyThemeToDOM(_mode: ThemeMode, resolved: "light" | "dark"): void {
  const root = document.documentElement;
  root.classList.remove("light", "dark");
  root.removeAttribute("data-theme");
  root.classList.add(resolved === "dark" ? "dark" : "light");
}

export function applyUIFontSize(px: number): void {
  const clamped = Math.min(22, Math.max(11, Math.round(px)));
  const scale = clamped / 14;
  document.documentElement.style.setProperty("--ui-font-scale", String(scale));
}

export function startSystemThemeListener(
  onSystemChange: (resolved: "light" | "dark") => void,
): void {
  stopSystemThemeListener();

  const mq = window.matchMedia(MEDIA_QUERY);
  const handler = (e: MediaQueryListEvent) => {
    onSystemChange(e.matches ? "dark" : "light");
  };

  mq.addEventListener("change", handler);
  mediaQueryCleanup = () => mq.removeEventListener("change", handler);
}

export function stopSystemThemeListener(): void {
  if (mediaQueryCleanup) {
    mediaQueryCleanup();
    mediaQueryCleanup = null;
  }
}

/** Migrate legacy stored values (e.g. "dim") into the supported ThemeMode set. */
export function parseThemeMode(value: string): ThemeMode {
  if (value === "light" || value === "dark" || value === "system") {
    return value;
  }
  if (value === "dim") {
    return "dark";
  }
  return "system";
}
