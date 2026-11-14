/** True when the UI runs on macOS. Synchronous (from the WebView itself), unlike
 *  `AppInfo.platform`, so keyboard handlers and labels can use it on the first render. */
export const isMac = (nav: Pick<Navigator, "platform" | "userAgent"> = navigator) =>
  /mac/i.test(nav.platform || nav.userAgent);
