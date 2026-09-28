// Typed wrappers around Tauri commands. Components use these, never `invoke` directly.
import { invoke } from "@tauri-apps/api/core";
import type {
  AppError,
  AppInfo,
  ExportFormat,
  ExportResult,
  KillReport,
  PortEntry,
  PortEvent,
  ProcessDetails,
  RemoteHost,
  Settings,
  Snapshot,
} from "./types";

export const PORTS_UPDATED = "ports-updated";
export const REQUEST_KILL = "request-kill";
export const SETTINGS_CHANGED = "settings-changed";
export const MODE_CHANGED = "mode-changed";

export const listPorts = () => invoke<PortEntry[]>("list_ports");
export const getProcessDetails = (pid: number) => invoke<ProcessDetails>("get_process_details", { pid });
export const killProcess = (pid: number, expectedStartMs: number, tree: boolean) =>
  invoke<KillReport>("kill_process", { pid, expectedStartMs, tree });
export const getSettings = () => invoke<Settings>("get_settings");
export const saveSettings = (settings: Settings) => invoke<Settings>("save_settings", { settings });
export const getAppInfo = () => invoke<AppInfo>("get_app_info");
export const HISTORY_UPDATED = "history-updated";

export const getHistory = () => invoke<PortEvent[]>("get_history");
export const clearHistory = () => invoke<void>("clear_history");
export const exportHistory = (format: ExportFormat, save: boolean) => invoke<ExportResult>("export_history", { format, save });

export const remoteList = (id: string) => invoke<Snapshot>("remote_list", { id });
export const remoteTest = (host: RemoteHost) => invoke<string>("remote_test", { host });
export const remoteKill = (id: string, pid: number, expectedStartMs: number, tree: boolean) =>
  invoke<KillReport>("remote_kill", { id, pid, expectedStartMs, tree });

/** `host` = remote host id (SSH view) or null for this computer. */
export const exportSnapshot = (format: ExportFormat, ids: string[], host: string | null, save: boolean) =>
  invoke<ExportResult>("export_snapshot", { format, ids, host, save });
export const openLogFolder =() => invoke<void>("open_log_folder");
export const windowShown = () => invoke<void>("window_shown");
export const relaunchAsAdmin = () => invoke<void>("relaunch_as_admin");

export function toAppError(e: unknown): AppError {
  if (e && typeof e === "object" && "message" in e && "kind" in e) return e as AppError;
  // JS errors: the message alone ("Invalid access key", not "Error: Invalid access key").
  const message = e instanceof Error ? e.message : String(e);
  return { kind: "other", message, detail: message };
}
