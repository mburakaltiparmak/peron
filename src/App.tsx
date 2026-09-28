import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { openUrl, revealItemInDir } from "@tauri-apps/plugin-opener";
import {
  exportHistory,
  exportSnapshot,
  getAppInfo,
  getSettings,
  MODE_CHANGED,
  relaunchAsAdmin,
  REQUEST_KILL,
  saveSettings,
  SETTINGS_CHANGED,
  toAppError,
  windowShown,
} from "./api";
import { ContactView } from "./components/ContactView";
import { DetailPanel } from "./components/DetailPanel";
import { FilterBar, LOCAL, type QuickFilter } from "./components/FilterBar";
import { HistoryView } from "./components/HistoryView";
import { KillConfirmDialog } from "./components/KillConfirmDialog";
import { isLongOpen, PortTable } from "./components/PortTable";
import { ServersDialog } from "./components/ServersDialog";
import { SettingsDialog } from "./components/SettingsDialog";
import { Toolbar, type Tab } from "./components/Toolbar";
import { DEVELOPER } from "./feedback";
import { filterPorts } from "./filter";
import { prereleaseLabel } from "./format";
import { useNow, usePorts } from "./hooks/usePorts";
import { useRemotePorts } from "./hooks/useRemotePorts";
import { dictionaries, errorText, I18nContext } from "./i18n";
import type { AppInfo, ExportFormat, ExportResult, ModeInfo, PortEntry, Settings } from "./types";
import { applyTheme } from "./theme";
import { checkForUpdate, dismissUpdate, type Update } from "./updates";


interface Toast {
  text: string;
  error?: boolean;
  action?: { label: string; run: () => void };
}

export default function App() {
  const [settings, setSettings] = useState<Settings | null>(null);
  const [info, setInfo] = useState<AppInfo | null>(null);
  const [mode, setMode] = useState<ModeInfo | null>(null);
  const [search, setSearch] = useState("");
  const [quick, setQuick] = useState<QuickFilter>("all");
  const [selectedId, setSelectedId] = useState<string | null>(null);
  const [killTarget, setKillTarget] = useState<PortEntry | null>(null);
  const [showSettings, setShowSettings] = useState(false);
  const [showServers, setShowServers] = useState(false);
  const [toast, setToast] = useState<Toast | null>(null);
  const [tab, setTab] = useState<Tab>("ports");
  const [host, setHost] = useState<string>(LOCAL);
  const [update, setUpdate] = useState<Update | null>(null);
  const searchRef = useRef<HTMLInputElement>(null);

  const local = usePorts(settings?.autoRefresh ?? true);
  const remoteId = host === LOCAL ? null : host;
  const remote = useRemotePorts(tab === "ports" ? remoteId : null, settings?.refreshIntervalSecs ?? 3, settings?.autoRefresh ?? true);
  const isRemote = remoteId !== null;
  const ports = isRemote ? (remote.snapshot?.entries ?? []) : local.ports;
  const loading = isRemote ? remote.loading : local.loading;
  const error = isRemote ? remote.error : local.error;
  const updatedAt = isRemote ? remote.updatedAt : local.updatedAt;
  const refresh = isRemote ? remote.refresh : local.refresh;
  const now = useNow(15_000);

  const lang = settings?.language ?? info?.language ?? "tr";
  const t = dictionaries[lang];
  const hostEntry = settings?.remoteHosts.find((h) => h.id === remoteId) ?? null;

  const loadInfo = useCallback(
    () =>
      getAppInfo().then((i) => {
        setInfo(i);
        setMode(i.mode);
      }),
    [],
  );

  useEffect(() => {
    getSettings().then((s) => {
      setSettings(s);
      applyTheme(s.theme);
    });
    loadInfo();
  }, [loadInfo]);

  // A removed host can't stay selected.
  useEffect(() => {
    if (settings && remoteId && !settings.remoteHosts.some((h) => h.id === remoteId)) setHost(LOCAL);
  }, [settings, remoteId]);

  // Direct-download builds only (the Store updates its own build); throttled to once a day.
  const checkUpdates = settings?.checkUpdates;
  useEffect(() => {
    if (!info || info.channel !== "direct" || !checkUpdates) return;
    let cancelled = false;
    checkForUpdate(info.version).then((u) => !cancelled && u && setUpdate(u));
    return () => {
      cancelled = true;
    };
  }, [info, checkUpdates]);

  // The window is created hidden; show it once the first real frame is ready (no blank flash).
  const shown = useRef(false);
  useEffect(() => {
    if (shown.current || !settings || !info || !local.loaded) return;
    shown.current = true;
    requestAnimationFrame(() => {
      const w = getCurrentWindow();
      w.show()
        .then(() => w.setFocus())
        .finally(() => windowShown());
    });
  }, [settings, info, local.loaded]);

  useEffect(() => {
    document.documentElement.lang = lang;
  }, [lang]);

  // Backend-driven changes (tray language menu, mode switches).
  useEffect(() => {
    const a = listen<Settings>(SETTINGS_CHANGED, (e) => {
      setSettings(e.payload);
      applyTheme(e.payload.theme);
      loadInfo();
    });
    const b = listen<ModeInfo>(MODE_CHANGED, (e) => setMode(e.payload));
    return () => {
      a.then((f) => f());
      b.then((f) => f());
    };
  }, [loadInfo]);

  useEffect(() => {
    if (!toast) return;
    const timer = setTimeout(() => setToast(null), toast.action ? 8000 : 5000);
    return () => clearTimeout(timer);
  }, [toast]);

  const updateSettings = useCallback(
    async (patch: Partial<Settings>) => {
      if (!settings) return;
      const next = { ...settings, ...patch };
      setSettings(next);
      try {
        setSettings(await saveSettings(next));
      } catch (e) {
        setToast({ text: errorText(t, toAppError(e)), error: true });
      }
    },
    [settings, t],
  );

  // Tray menu → open the confirmation dialog for that (local) entry.
  const localRef = useRef(local.ports);
  localRef.current = local.ports;
  const localRefresh = local.refresh;
  useEffect(() => {
    const unlisten = listen<string>(REQUEST_KILL, async (ev) => {
      let entry = localRef.current.find((p) => p.id === ev.payload);
      if (!entry) entry = (await localRefresh())?.find((p) => p.id === ev.payload);
      if (entry) {
        setTab("ports");
        setHost(LOCAL);
        setKillTarget(entry);
      } else setToast({ text: t.portGone });
    });
    return () => {
      unlisten.then((f) => f());
    };
  }, [localRefresh, t]);

  const longOpenMs = (settings?.reminderThresholdMinutes ?? 240) * 60_000;
  const nowForRows = isRemote && remote.snapshot ? remote.snapshot.generatedAtMs + (now - (remote.updatedAt ?? now)) : now;

  // View + system + UDP + search, before the quick filter (so chip counts stay meaningful).
  const base = useMemo(() => (settings ? filterPorts(ports, settings, search) : []), [ports, settings, search]);

  const counts = useMemo(
    () => ({
      all: base.length,
      dev: base.filter((e) => e.isDev).length,
      long: base.filter((e) => isLongOpen(e, nowForRows, longOpenMs)).length,
    }),
    [base, nowForRows, longOpenMs],
  );

  const visible = useMemo(() => {
    if (quick === "dev") return base.filter((e) => e.isDev);
    if (quick === "long") return base.filter((e) => isLongOpen(e, nowForRows, longOpenMs));
    return base;
  }, [base, quick, nowForRows, longOpenMs]);

  const selected = visible.find((p) => p.id === selectedId) ?? ports.find((p) => p.id === selectedId) ?? null;
  const samePid = (pid: number) => ports.filter((p) => p.pid === pid);

  // Keyboard shortcuts. F5/Ctrl+R would reload the WebView, so they are always intercepted.
  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      const typing =
        e.target instanceof HTMLInputElement || e.target instanceof HTMLSelectElement || e.target instanceof HTMLTextAreaElement;
      if (e.key === "F5" || (e.ctrlKey && e.key.toLowerCase() === "r")) {
        e.preventDefault();
        if (tab === "ports") refresh();
      } else if (tab !== "ports") {
        return;
      } else if (e.ctrlKey && e.key.toLowerCase() === "f") {
        e.preventDefault();
        searchRef.current?.focus();
        searchRef.current?.select();
      } else if (e.key === "Delete" && !typing && selected && !selected.isProtected && !killTarget) {
        setKillTarget(selected);
      } else if (e.key === "Escape" && !killTarget && !showSettings && !showServers) {
        if (typing && search) setSearch("");
        else setSelectedId(null);
      }
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [refresh, selected, killTarget, showSettings, showServers, search, tab]);

  const afterExport = async (result: Promise<ExportResult>) => {
    try {
      const r = await result;
      if (r.path) {
        const path = r.path;
        setToast({ text: t.exported(r.fileName), action: { label: t.showInFolder, run: () => revealItemInDir(path) } });
      } else if (r.text != null) {
        // The browser refuses when the document isn't focused; say so instead of a raw DOMException.
        const ok = await navigator.clipboard.writeText(r.text).then(
          () => true,
          () => false,
        );
        setToast(ok ? { text: t.copied } : { text: t.copyFailed, error: true });
      }
    } catch (e) {
      setToast({ text: errorText(t, toAppError(e)), error: true });
    }
  };

  // Export exactly what's visible: every socket behind the (grouped) rows on screen.
  const onExportPorts = (format: ExportFormat, save: boolean) => {
    const keys = new Set(visible.map((r) => `${r.protocol}|${r.pid}|${r.localPort}|${r.state}`));
    const ids = ports.filter((p) => keys.has(`${p.protocol}|${p.pid}|${p.localPort}|${p.state}`)).map((p) => p.id);
    afterExport(exportSnapshot(format, ids, remoteId, save));
  };

  if (!settings || !info) return <div className="empty">{t.loading}</div>;

  const listeningCount = visible.filter((e) => e.state === "LISTEN").length;
  const modeLabel = isRemote
    ? remote.snapshot
      ? t.remoteStatus(hostEntry?.name ?? remote.snapshot.host, remote.snapshot.os)
      : (hostEntry?.name ?? "")
    : !settings.autoRefresh
      ? t.modeManual
      : mode?.mode === "active"
        ? t.modeLive(mode.intervalSecs)
        : t.modeSlow(mode?.intervalSecs ?? settings.refreshIntervalSecs);
  const elevated = isRemote ? (remote.snapshot?.elevated ?? true) : info.elevated;

  return (
    <I18nContext.Provider value={t}>
      <div className="app">
        <Toolbar
          ref={searchRef}
          tab={tab}
          onTab={setTab}
          prerelease={prereleaseLabel(info.version)}
          settings={settings}
          search={search}
          loading={loading}
          showAdmin={info.canElevate && !info.elevated}
          host={host}
          onSearch={setSearch}
          onChange={updateSettings}
          onRefresh={refresh}
          onExport={onExportPorts}
          onOpenSettings={() => setShowSettings(true)}
          onRelaunchAdmin={() => relaunchAsAdmin().catch((e) => setToast({ text: errorText(t, toAppError(e)), error: true }))}
        />
        {tab === "ports" && (
          <FilterBar
            settings={settings}
            filter={quick}
            counts={counts}
            onFilter={setQuick}
            onChange={updateSettings}
            host={host}
            onHost={(id) => {
              setHost(id);
              setSelectedId(null);
            }}
            onManageHosts={() => setShowServers(true)}
          />
        )}

        {update && (
          <div className="banner update" role="status">
            <span>{t.updateAvailable(update.version)}</span>
            <button className="link" onClick={() => openUrl(update.url)}>{t.updateDownload}</button>
            <button
              className="link muted"
              onClick={() => {
                dismissUpdate(update.version);
                setUpdate(null);
              }}
            >
              {t.updateDismiss}
            </button>
          </div>
        )}

        {tab === "ports" && error && (
          <div className="banner error">
            {errorText(t, error)}
            {isRemote && error.code === "cliMissing" && (
              <button className="link" onClick={() => setShowServers(true)}>{t.manageServers}</button>
            )}
          </div>
        )}

        {tab === "contact" ? (
          <main className="scroll">
            <ContactView info={info} lang={lang} onSent={(text) => setToast({ text })} />
          </main>
        ) : tab === "history" ? (
          <main className="scroll">
            <HistoryView hideSystem={settings.hideSystem} onExport={(f, save) => afterExport(exportHistory(f, save))} />
          </main>
        ) : (
          <main className={selected ? "with-detail" : ""}>
            {!local.loaded || (isRemote && !remote.snapshot && !remote.error) ? (
              <div className="skeleton" aria-busy="true">
                {Array.from({ length: 8 }, (_, i) => <div key={i} className="skeleton-row" />)}
              </div>
            ) : (
              <PortTable
                ports={visible}
                now={nowForRows}
                longOpenMs={longOpenMs}
                selectedId={selectedId}
                onSelect={(e) => setSelectedId(e.id)}
                onKill={setKillTarget}
                onClearFilters={() => {
                  setSearch("");
                  setQuick("all");
                }}
              />
            )}
            {selected && (
              <DetailPanel
                entry={selected}
                samePid={samePid(selected.pid)}
                now={nowForRows}
                remote={isRemote}
                onClose={() => setSelectedId(null)}
                onKill={setKillTarget}
              />
            )}
          </main>
        )}

        <footer className="status">
          <span>{tab === "ports" && `${t.rows(visible.length)} · ${t.listening(listeningCount)}`}</span>
          <span className="muted">
            <span className={`dot ${!isRemote && mode?.mode === "active" && settings.autoRefresh ? "live" : ""}`} /> {modeLabel}
            {updatedAt ? ` · ${t.updatedAt(new Date(updatedAt).toLocaleTimeString(t.locale))}` : ""}
            {!elevated && (
              <span title={info.platform === "linux" || isRemote ? t.limitedLinuxTitle : t.adminTitle}> · {t.limited}</span>
            )}
            {` · v${info.version} · `}
            {t.developer}:{" "}
            <button className="link credit" onClick={() => openUrl(DEVELOPER.url)} title={DEVELOPER.url}>
              {DEVELOPER.name}
            </button>
          </span>
        </footer>

        {killTarget && (
          <KillConfirmDialog
            entry={killTarget}
            samePid={samePid(killTarget.pid)}
            remote={hostEntry && isRemote ? { id: hostEntry.id, name: hostEntry.name } : null}
            onCancel={() => setKillTarget(null)}
            onDone={(text) => {
              setKillTarget(null);
              setToast({ text });
              if (isRemote || !settings.autoRefresh) refresh();
            }}
          />
        )}

        {showSettings && (
          <SettingsDialog
            settings={settings}
            language={lang}
            channel={info.channel}
            onCancel={() => setShowSettings(false)}
            onSave={async (s) => {
              const saved = await saveSettings(s);
              setSettings(saved);
              applyTheme(saved.theme);
              setShowSettings(false);
            }}
          />
        )}

        {showServers && (
          <ServersDialog
            hosts={settings.remoteHosts}
            version={info.version}
            onClose={() => setShowServers(false)}
            onSave={async (remoteHosts) => {
              await updateSettings({ remoteHosts });
              setShowServers(false);
              const added = remoteHosts.find((h) => !settings.remoteHosts.some((o) => o.id === h.id));
              if (added) setHost(added.id);
            }}
          />
        )}

        {toast && (
          <div className={`toast ${toast.error ? "error" : ""}`} role="status">
            {toast.text}
            {toast.action && (
              <button
                className="link toast-action"
                onClick={() => {
                  toast.action?.run();
                  setToast(null);
                }}
              >
                {toast.action.label}
              </button>
            )}
          </div>
        )}
      </div>
    </I18nContext.Provider>
  );
}
