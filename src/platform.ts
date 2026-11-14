/** True when the UI runs on macOS. Synchronous (from the WebView itself), unlike
 *  `AppInfo.platform`, so keyboard handlers and labels can use it on the first render. */
export const isMac = (nav: Pick<Navigator, "platform" | "userAgent"> = navigator) =>
  /mac/i.test(nav.platform || nav.userAgent);

/** Marks the document so `styles.macos.css` (all rules under `[data-platform="macos"]`) applies.
 *  Windows and Linux keep the base styles untouched. */
export function markPlatform(root: HTMLElement = document.documentElement, nav?: Pick<Navigator, "platform" | "userAgent">) {
  if (isMac(nav)) root.dataset.platform = "macos";
}

/** macOS: the title bar is hidden, so the toolbar's empty areas drag the window. Elsewhere the
 *  attribute is left out (the native title bar does that, and the permission is macOS-only). */
export const dragRegion = (): { "data-tauri-drag-region"?: "" } => (isMac() ? { "data-tauri-drag-region": "" } : {});
