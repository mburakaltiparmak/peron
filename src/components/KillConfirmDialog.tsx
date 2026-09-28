import { useEffect, useRef, useState } from "react";
import { useDialogFocus } from "../hooks/useDialogFocus";
import { getProcessDetails, killProcess, relaunchAsAdmin, remoteKill, toAppError } from "../api";
import { errorText, useT } from "../i18n";
import type { AppError, PortEntry, ProcRef } from "../types";
import { FolderIcon } from "./Icons";

interface Props {
  entry: PortEntry;
  samePid: PortEntry[];
  /** Set when the row comes from a server (SSH view): the process is ended there. */
  remote?: { id: string; name: string } | null;
  onCancel: () => void;
  onDone: (message: string) => void;
}

/** The only path to killing a process: explicit user confirmation. */
export function KillConfirmDialog({ entry: e, samePid, remote = null, onCancel, onDone }: Props) {
  const t = useT();
  const [children, setChildren] = useState<ProcRef[]>([]);
  const dialogRef = useRef<HTMLDivElement>(null);
  useDialogFocus(dialogRef);
  const [tree, setTree] = useState(false);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<AppError | null>(null);

  useEffect(() => {
    if (remote) return; // the local process tree says nothing about a server's PID
    getProcessDetails(e.pid).then((d) => setChildren(d.children)).catch(() => {});
  }, [e.pid, remote]);

  useEffect(() => {
    const onKey = (ev: KeyboardEvent) => {
      if (ev.key === "Escape" && !busy) {
        ev.stopPropagation();
        onCancel();
      }
    };
    window.addEventListener("keydown", onKey, true);
    return () => window.removeEventListener("keydown", onKey, true);
  }, [busy, onCancel]);

  const ports = [...new Set(samePid.map((p) => `${p.protocol} ${p.localPort}`))];
  const project = e.project;

  const confirm = async () => {
    setBusy(true);
    setError(null);
    try {
      const report = remote
        ? await remoteKill(remote.id, e.pid, e.processStartMs, tree)
        : await killProcess(e.pid, e.processStartMs, tree);
      onDone(t.killed(e.processName, report.killed.length, e.localPort, report.failed.length));
    } catch (err) {
      setError(toAppError(err));
      setBusy(false);
    }
  };

  return (
    <div className="backdrop" onClick={() => !busy && onCancel()}>
      <div ref={dialogRef} className="dialog" role="alertdialog" aria-modal="true" aria-labelledby="kill-title" onClick={(ev) => ev.stopPropagation()}>
        <h2 id="kill-title">{t.killTitle(e.localPort)}</h2>
        <p>{remote ? t.killBodyRemote(e.processName, e.pid, remote.name) : t.killBody(e.processName, e.pid)}</p>

        {e.cmdline && <pre className="cmd">{e.cmdline}</pre>}
        {e.cwd && (
          <p className="muted small">
            <FolderIcon size={12} /> {project && <strong>{project}</strong>} <span className="mono">{e.cwd}</span>
          </p>
        )}

        {ports.length > 1 && <p className="note">{t.alsoCloses(ports.join(", "))}</p>}

        {children.length > 0 && (
          <label className="check">
            <input type="checkbox" checked={tree} onChange={(ev) => setTree(ev.target.checked)} />
            {t.killTree(
              children.length,
              children.slice(0, 4).map((c) => c.name).join(", ") + (children.length > 4 ? "…" : ""),
            )}
          </label>
        )}

        {error && (
          <div className="error">
            {errorText(t, error)}
            {error.kind === "accessDenied" && !remote && (
              <button className="link" onClick={() => relaunchAsAdmin().catch((x) => setError(toAppError(x)))}>
                {t.relaunchAdmin}
              </button>
            )}
          </div>
        )}

        <div className="dialog-actions">
          <button onClick={onCancel} disabled={busy} autoFocus>{t.cancel}</button>
          <button className="danger" onClick={confirm} disabled={busy}>
            {busy ? t.terminating : t.terminate}
          </button>
        </div>
      </div>
    </div>
  );
}
