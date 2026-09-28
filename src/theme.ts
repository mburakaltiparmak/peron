import { getCurrentWindow } from "@tauri-apps/api/window";
import type { Theme } from "./types";

/**
 * Applies a theme to the page (CSS tokens via `data-theme`) and to the native window, so the
 * Windows title bar matches. "system" follows the OS for both.
 */
export function applyTheme(theme: Theme) {
  const root = document.documentElement;
  if (theme === "system") root.removeAttribute("data-theme");
  else root.setAttribute("data-theme", theme);
  // Cosmetic only: never break the UI over the title bar (or outside Tauri, e.g. in tests).
  try {
    getCurrentWindow()
      .setTheme(theme === "system" ? null : theme)
      .catch(() => {});
  } catch {
    /* no Tauri window */
  }
}
