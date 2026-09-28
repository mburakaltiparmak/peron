import { useEffect, useMemo, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import { clearHistory, getHistory, HISTORY_UPDATED } from "../api";
import { formatDateTime } from "../format";
import { useT } from "../i18n";
import type { ExportFormat, PortEvent } from "../types";
import { ExportMenu } from "./ExportMenu";
import { FolderIcon, SearchIcon } from "./Icons";

interface Props {
  hideSystem: boolean;
  onExport: (format: ExportFormat, save: boolean) => void;
}

/** Local port open/close timeline (newest first), recorded by the backend monitor. */
export function HistoryView({ hideSystem, onExport }: Props) {
  const t = useT();
  const [events, setEvents] = useState<PortEvent[]>([]);
  const [search, setSearch] = useState("");
  const [confirming, setConfirming] = useState(false);

  useEffect(() => {
    getHistory().then(setEvents).catch(() => {});
    const un = listen<PortEvent[]>(HISTORY_UPDATED, (e) => setEvents((prev) => [...[...e.payload].reverse(), ...prev]));
    return () => {
      un.then((f) => f());
    };
  }, []);

  const visible = useMemo(() => {
    const q = search.trim().toLowerCase();
    return events.filter(
      (e) =>
        (!hideSystem || !e.isSystem) &&
        (!q || [String(e.port), e.process, e.project ?? "", String(e.pid), e.addrs.join(" ")].join(" ").toLowerCase().includes(q)),
    );
  }, [events, search, hideSystem]);

  return (
    <div className="history">
      <div className="history-bar">
        <label className="search">
          <SearchIcon size={15} />
          <input type="search" aria-label={t.searchLabel} placeholder={t.searchPlaceholder} value={search} onChange={(e) => setSearch(e.target.value)} />
        </label>
        <span className="muted small">{t.historyCount(visible.length)}</span>
        <div className="spacer" />
        <ExportMenu disabled={events.length === 0} onExport={onExport} />
        {confirming ? (
          <span className="confirm-inline">
            {t.confirmClear}
            <button
              className="danger small"
              onClick={() => {
                clearHistory().then(() => setEvents([]));
                setConfirming(false);
              }}
            >
              {t.yes}
            </button>
            <button className="small" autoFocus onClick={() => setConfirming(false)}>{t.no}</button>
          </span>
        ) : (
          <button disabled={events.length === 0} onClick={() => setConfirming(true)}>{t.clearHistory}</button>
        )}
      </div>
      <p className="muted small history-note">{t.historyNote}</p>

      {visible.length === 0 ? (
        <div className="empty"><p className="muted">{t.historyEmpty}</p></div>
      ) : (
        <div className="table-wrap">
          <table className="ports history-table">
            <thead>
              <tr>
                <th>{t.colTime}</th>
                <th>{t.colEvent}</th>
                <th className="num">{t.colPort}</th>
                <th>{t.colProcess}</th>
                <th>{t.colAddress}</th>
              </tr>
            </thead>
            <tbody>
              {visible.map((e, i) => (
                <tr key={`${e.atMs}-${e.kind}-${e.pid}-${e.port}-${i}`}>
                  <td className="nowrap">{formatDateTime(e.atMs, t.locale)}</td>
                  <td>
                    <span className={`badge ${e.kind === "opened" ? "local" : "sys"}`}>{e.kind === "opened" ? t.evOpened : t.evClosed}</span>
                  </td>
                  <td className="num port">{e.port}</td>
                  <td>
                    <div className="proc">
                      {e.process}
                      {e.isDev && <span className="badge dev">{t.badgeDev}</span>}
                    </div>
                    <div className="muted small sub">
                      PID {e.pid}
                      {e.project && (
                        <span className="project"><FolderIcon size={12} /> {e.project}</span>
                      )}
                    </div>
                  </td>
                  <td>
                    <span className="mono">{e.addrs.join(", ")}</span>
                    {e.exposed && <span className="badge exposed" title={t.badgeExposedTitle}>{t.exposedBadge}</span>}
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      )}
    </div>
  );
}
