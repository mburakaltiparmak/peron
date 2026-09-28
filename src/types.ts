// Mirrors the Rust DTOs (serde camelCase).

export type Protocol = "TCP" | "UDP";

export interface PortEntry {
  id: string;
  protocol: Protocol;
  ipVersion: 4 | 6;
  localAddr: string;
  localPort: number;
  remoteAddr: string | null;
  remotePort: number | null;
  /** TCP state, or "BOUND" for UDP. */
  state: string;
  pid: number;
  processName: string;
  exePath: string | null;
  cmdline: string | null;
  cwd: string | null;
  /** Last folder of cwd when meaningful (computed in peron-core). */
  project: string | null;
  parentPid: number | null;
  parentName: string | null;
  openedAtMs: number | null;
  openedAtSource: "socket" | "process";
  processStartMs: number;
  cpuPercent: number;
  memoryBytes: number;
  diskReadBytes: number;
  diskWrittenBytes: number;
  isSystem: boolean;
  isProtected: boolean;
  isDev: boolean;
  isLocalhostOnly: boolean;
  accessible: boolean;
}

/** A table row: one or more sockets of the same process/protocol/port (e.g. IPv4 + IPv6). */
export interface PortRow extends PortEntry {
  addrs: string[];
  ipVersions: number[];
}

export interface ProcRef {
  pid: number;
  name: string;
  exe: string | null;
}

export interface ProcessDetails {
  pid: number;
  parentChain: ProcRef[];
  children: ProcRef[];
}

export interface KillReport {
  killed: number[];
  failed: { pid: number; name: string; message: string }[];
}

export type ViewMode = "listen" | "all";
export type ExportFormat = "txt" | "csv" | "json";

/** peron-core export::Snapshot (schema v1) — also what `peron-cli list --json` prints. */
export interface Snapshot {
  schemaVersion: number;
  tool: string;
  version: string;
  host: string;
  os: string;
  generatedAtMs: number;
  elevated: boolean;
  entries: PortEntry[];
}

export interface PortEvent {
  atMs: number;
  kind: "opened" | "closed";
  port: number;
  pid: number;
  process: string;
  project: string | null;
  addrs: string[];
  exposed: boolean;
  isSystem: boolean;
  isDev: boolean;
}

/** An SSH target for the remote view. */
export interface RemoteHost {
  id: string;
  name: string;
  /** user@host, host or an ~/.ssh/config alias */
  target: string;
  port: number | null;
}

export type RemoteErrorCode =
  | "sshMissing"
  | "invalidHost"
  | "auth"
  | "hostKey"
  | "unreachable"
  | "cliMissing"
  | "badOutput"
  | "incompatibleVersion"
  | "other";

export interface ExportResult {
  fileName: string;
  path: string | null;
  text: string | null;
}
export type Lang = "tr" | "en";
export type Theme = "system" | "light" | "dark";

export interface Settings {
  /** null = never chosen in the app; the backend resolves installer/Windows language. */
  language: Lang | null;
  theme: Theme;
  viewMode: ViewMode;
  hideSystem: boolean;
  showUdp: boolean;
  autoRefresh: boolean;
  refreshIntervalSecs: number;
  reminderEnabled: boolean;
  reminderThresholdMinutes: number;
  reminderRepeatMinutes: number;
  closeToTray: boolean;
  autostart: boolean;
  checkUpdates: boolean;
  alertExposed: boolean;
  remoteHosts: RemoteHost[];
}

export interface AppError {
  kind: "accessDenied" | "notFound" | "protected" | "changed" | "rateLimited" | "remote" | "other";
  /** Turkish fallback text; the UI translates by `kind`. */
  message: string;
  detail?: string | null;
  /** Set when kind = "remote". */
  code?: RemoteErrorCode | null;
}

export type Mode = "active" | "passive" | "background" | "idle";

export interface ModeInfo {
  mode: Mode;
  intervalSecs: number;
}

export interface AppInfo {
  /** Effective language: Settings choice → installer choice → OS language → English. */
  language: Lang;
  elevated: boolean;
  /** Windows only: relaunch through UAC. On Linux the user runs Peron as root instead. */
  canElevate: boolean;
  /** "store" = MSIX from Microsoft Store (Store updates it); "direct" = site/GitHub packages. */
  channel: "store" | "direct";
  platform: "windows" | "linux";
  /** e.g. "Windows 11 Home 10.0.26200" or "Ubuntu 24.04". */
  os: string;
  version: string;
  mode: ModeInfo;
}
