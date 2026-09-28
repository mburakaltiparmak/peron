// Pure list logic for the Ports tab (unit-tested in filter.test.ts).
import type { PortEntry, PortRow, Settings } from "./types";

/** Every whitespace-separated term must appear in port, process, parent, command, cwd, project, address or PID. */
export function matches(e: PortEntry, q: string): boolean {
  if (!q) return true;
  const hay = [
    String(e.localPort),
    e.processName,
    e.parentName,
    e.cmdline,
    e.cwd,
    e.project,
    e.localAddr,
    e.remoteAddr,
    String(e.pid),
  ]
    .filter(Boolean)
    .join(" ")
    .toLowerCase();
  return q.toLowerCase().split(/\s+/).every((t) => hay.includes(t));
}

/** Listen view: merge sockets of the same process + protocol + port into one row (IPv4 + IPv6, SO_REUSEADDR duplicates). */
export function groupRows(entries: PortEntry[], group: boolean): PortRow[] {
  if (!group) return entries.map((e) => ({ ...e, addrs: [e.localAddr], ipVersions: [e.ipVersion] }));
  const rows = new Map<string, PortRow>();
  for (const e of entries) {
    const key = `${e.protocol}|${e.pid}|${e.localPort}`;
    const row = rows.get(key);
    if (!row) {
      rows.set(key, { ...e, addrs: [e.localAddr], ipVersions: [e.ipVersion] });
      continue;
    }
    if (!row.addrs.includes(e.localAddr)) row.addrs.push(e.localAddr);
    if (!row.ipVersions.includes(e.ipVersion)) row.ipVersions.push(e.ipVersion);
    row.isLocalhostOnly &&= e.isLocalhostOnly;
    if (e.openedAtMs && (!row.openedAtMs || e.openedAtMs < row.openedAtMs)) row.openedAtMs = e.openedAtMs;
  }
  return [...rows.values()];
}

/** View mode + system/UDP toggles + search, then grouping (listen view only). */
export function filterPorts(
  ports: PortEntry[],
  s: Pick<Settings, "viewMode" | "showUdp" | "hideSystem">,
  search: string,
): PortRow[] {
  const filtered = ports.filter(
    (e) =>
      (s.viewMode === "all" || e.state === "LISTEN" || e.protocol === "UDP") &&
      (s.showUdp || e.protocol !== "UDP") &&
      (!s.hideSystem || !e.isSystem) &&
      matches(e, search.trim()),
  );
  return groupRows(filtered, s.viewMode === "listen");
}
