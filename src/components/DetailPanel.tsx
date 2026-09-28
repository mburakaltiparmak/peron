import { useEffect, useState } from "react";
import { revealItemInDir } from "@tauri-apps/plugin-opener";
import { getProcessDetails } from "../api";
import { formatBytes, formatCpu, formatDateTime } from "../format";
import { useT } from "../i18n";
import type { PortEntry, PortRow, ProcessDetails } from "../types";
import { CloseIcon, FolderIcon, PowerIcon } from "./Icons";

interface Props {
  entry: PortEntry | PortRow;
  samePid: PortEntry[];
  now: number;
  /** Row from a server (SSH view): no local process-tree lookup, no "show in folder". */
  remote?: boolean;
  onClose: () => void;
  onKill: (e: PortEntry) => void;
}

export function DetailPanel({ entry: e, samePid, now, remote = false, onClose, onKill }: Props) {
  const t = useT();
  const [details, setDetails] = useState<ProcessDetails | null>(null);

  useEffect(() => {
    let cancelled = false;
    setDetails(null);
    if (remote) return;
    getProcessDetails(e.pid)
      .then((d) => !cancelled && setDetails(d))
      .catch(() => {});
    return () => {
      cancelled = true;
    };
  }, [e.pid, remote]);

  const otherPorts = samePid.filter((p) => p.localPort !== e.localPort || p.protocol !== e.protocol);
  const addrs = "addrs" in e ? e.addrs : [e.localAddr];
  const project = e.project;

  return (
    <aside className="detail">
      <div className="detail-head">
        <div>
          <h2>
            {e.processName} <span className="muted">:{e.localPort}</span>
          </h2>
          <div className="muted small">PID {e.pid} · {e.protocol} · {e.state}</div>
        </div>
        <button className="icon" onClick={onClose} aria-label={t.closePanel} title={`${t.closePanel} (Esc)`}>
          <CloseIcon size={15} />
        </button>
      </div>

      {remote ? <p className="note">{t.remoteNoTree}</p> : !e.accessible && <p className="note">{t.notAccessible}</p>}

      <dl>
        {project && (
          <>
            <dt>{t.project}</dt>
            <dd className="project-name"><FolderIcon size={14} /> {project}</dd>
          </>
        )}

        <dt>{t.openedAt}</dt>
        <dd>
          {e.openedAtMs ? `${formatDateTime(e.openedAtMs, t.locale)} (${t.ago(t.duration(now - e.openedAtMs))})` : "—"}
          {e.openedAtSource === "process" && <div className="muted small">{t.fallbackStartNote}</div>}
        </dd>

        <dt>{t.processStart}</dt>
        <dd>{e.processStartMs ? formatDateTime(e.processStartMs, t.locale) : "—"}</dd>

        <dt>{t.parentChain}</dt>
        <dd>
          {details && details.parentChain.length > 0 ? (
            <ol className="chain">
              {[...details.parentChain].reverse().map((p) => (
                <li key={p.pid}>{p.name} <span className="muted small">({p.pid})</span></li>
              ))}
              <li><strong>{e.processName}</strong> <span className="muted small">({e.pid})</span></li>
            </ol>
          ) : (
            e.parentName ?? "—"
          )}
        </dd>

        {details && details.children.length > 0 && (
          <>
            <dt>{t.children}</dt>
            <dd>{details.children.map((c) => `${c.name} (${c.pid})`).join(", ")}</dd>
          </>
        )}

        <dt>{t.cwd}</dt>
        <dd className="mono break">{e.cwd ?? "—"}</dd>

        <dt>{t.cmdline}</dt>
        <dd className="mono break">{e.cmdline ?? "—"}</dd>

        <dt>{t.exePath}</dt>
        <dd className="mono break">
          {e.exePath ?? "—"}
          {e.exePath && !remote && (
            <div>
              <button className="link" onClick={() => revealItemInDir(e.exePath!)}>{t.reveal}</button>
            </div>
          )}
        </dd>

        <dt>{t.resources}</dt>
        <dd>
          CPU {formatCpu(e.cpuPercent)} · RAM {formatBytes(e.memoryBytes)}
          <div className="muted small">{t.diskIo(formatBytes(e.diskReadBytes), formatBytes(e.diskWrittenBytes))}</div>
        </dd>

        <dt>{t.address}</dt>
        <dd className="mono">
          {addrs.map((a) => `${a}:${e.localPort}`).join(", ")}
          {e.remoteAddr && <> → {e.remoteAddr}:{e.remotePort}</>}
        </dd>

        {otherPorts.length > 0 && (
          <>
            <dt>{t.otherPorts}</dt>
            <dd>{[...new Set(otherPorts.map((p) => `${p.protocol} ${p.localPort}`))].join(", ")}</dd>
          </>
        )}
      </dl>

      <button className="danger wide" disabled={e.isProtected} onClick={() => onKill(e)}>
        <PowerIcon size={14} /> {e.isProtected ? t.protectedProc : t.closePort}
      </button>
    </aside>
  );
}
