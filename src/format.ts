export function formatBytes(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`;
  const units = ["KB", "MB", "GB", "TB"];
  let v = bytes / 1024;
  let i = 0;
  while (v >= 1024 && i < units.length - 1) {
    v /= 1024;
    i++;
  }
  return `${v.toFixed(v < 10 ? 1 : 0)} ${units[i]}`;
}

export function formatDateTime(ms: number, locale: string): string {
  return new Date(ms).toLocaleString(locale, { dateStyle: "short", timeStyle: "medium" });
}

/** "0.1.0-beta.1" → "BETA"; stable versions → null. */
export function prereleaseLabel(version: string): string | null {
  const tag = version.split("-")[1]?.split(".")[0];
  return tag ? tag.toUpperCase() : null;
}

export function formatCpu(pct: number): string {
  return pct < 0.1 ? "0%" : `${pct.toFixed(1)}%`;
}
