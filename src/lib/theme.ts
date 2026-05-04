export type ThemeMode = "light" | "dark" | "dim" | "system";

export interface AccentColor {
  name: string;
  value: string;
  hsl: string;
  hslForeground: string;
}

export const ACCENT_COLORS: AccentColor[] = [
  { name: "Indigo", value: "#6366F1", hsl: "238.7 83.5% 66.7%", hslForeground: "0 0% 100%" },
  { name: "Violet", value: "#8B5CF6", hsl: "258.3 89.8% 66.3%", hslForeground: "0 0% 100%" },
  { name: "Blue", value: "#3B82F6", hsl: "217.2 91.2% 59.8%", hslForeground: "0 0% 100%" },
  { name: "Cyan", value: "#06B6D4", hsl: "187.9 85.7% 42.7%", hslForeground: "0 0% 100%" },
  { name: "Emerald", value: "#10B981", hsl: "160.1 84.1% 39.4%", hslForeground: "0 0% 100%" },
  { name: "Amber", value: "#F59E0B", hsl: "37.7 92.1% 50.2%", hslForeground: "0 0% 0%" },
  { name: "Rose", value: "#F43F5E", hsl: "349.7 89.2% 60.2%", hslForeground: "0 0% 100%" },
  { name: "Zinc", value: "#71717A", hsl: "240 3.8% 46.1%", hslForeground: "0 0% 100%" },
];

const MEDIA_QUERY = "(prefers-color-scheme: dark)";

let mediaQueryCleanup: (() => void) | null = null;

function getSystemTheme(): "light" | "dark" {
  if (typeof window === "undefined") return "light";
  return window.matchMedia(MEDIA_QUERY).matches ? "dark" : "light";
}

export function resolveTheme(mode: ThemeMode): "light" | "dark" {
  if (mode === "system") return getSystemTheme();
  if (mode === "dim") return "dark";
  return mode;
}

export function applyThemeToDOM(mode: ThemeMode, resolved: "light" | "dark"): void {
  const root = document.documentElement;
  root.classList.remove("light", "dark");

  if (mode === "dim") {
    root.classList.add("dark");
    root.dataset.theme = "dim";
    return;
  }

  root.removeAttribute("data-theme");
  root.classList.add(resolved === "dark" ? "dark" : "light");
}

export function applyAccentColor(hexColor: string): void {
  const accent = ACCENT_COLORS.find((c) => c.value === hexColor);
  if (!accent) return;

  const root = document.documentElement;
  root.style.setProperty("--primary", `hsl(${accent.hsl})`);
  root.style.setProperty("--primary-foreground", `hsl(${accent.hslForeground})`);
  root.style.setProperty("--ring", `hsl(${accent.hsl})`);
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
