import type { Config } from "./types";

export function resolveTheme(theme: string): "dark" | "light" {
  if (theme === "light") return "light";
  if (theme === "dark") return "dark";
  return window.matchMedia?.("(prefers-color-scheme: light)")?.matches
    ? "light"
    : "dark";
}

export function applyAppearance(config: Config): void {
  const theme = resolveTheme(config.theme);
  const root = document.documentElement;
  root.dataset.theme = theme;
  root.classList.toggle("dark", theme === "dark");
  root.dataset.density = config.density === "compact" ? "compact" : "comfortable";
}

export function watchSystemTheme(callback: () => void): () => void {
  const media = window.matchMedia?.("(prefers-color-scheme: light)");
  if (!media) return () => {};
  media.addEventListener("change", callback);
  return () => media.removeEventListener("change", callback);
}
