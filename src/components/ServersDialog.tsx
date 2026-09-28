import { useRef, useState } from "react";
import { remoteTest, toAppError } from "../api";
import { useDialogFocus } from "../hooks/useDialogFocus";
import { errorText, useT } from "../i18n";
import type { RemoteHost } from "../types";

/** Same rule as peron_core::remote::is_valid_target (the backend re-checks). */
export function isValidTarget(t: string): boolean {
  return /^[A-Za-z0-9._@:-]{1,255}$/.test(t) && !t.startsWith("-") && (t.match(/@/g) ?? []).length <= 1;
}

/** Pinned to this app's version: GitHub's /releases/latest/ skips pre-releases (betas). */
export const cliInstall = (version: string) =>
  `curl -fsSL -o /tmp/peron-cli https://github.com/mburakaltiparmak/peron/releases/download/v${version}/peron-cli-x86_64-linux && sudo install -m 755 /tmp/peron-cli /usr/local/bin/peron-cli`;

interface Props {
  hosts: RemoteHost[];
  version: string;
  onSave: (hosts: RemoteHost[]) => Promise<void>;
  onClose: () => void;
}

export function ServersDialog({ hosts, version, onSave, onClose }: Props) {
  const t = useT();
  const ref = useRef<HTMLDivElement>(null);
  useDialogFocus(ref);
  const [list, setList] = useState<RemoteHost[]>(hosts);
  const [name, setName] = useState("");
  const [target, setTarget] = useState("");
  const [port, setPort] = useState("");
  const [status, setStatus] = useState<{ text: string; error?: boolean } | null>(null);
  const [busy, setBusy] = useState(false);

  const draft = (): RemoteHost | null => {
    const tgt = target.trim();
    if (!isValidTarget(tgt)) {
      setStatus({ text: t.invalidTarget, error: true });
      return null;
    }
    const p = port.trim() ? Number(port) : null;
    if (p !== null && !(Number.isInteger(p) && p > 0 && p < 65536)) {
      setStatus({ text: t.invalidTarget, error: true });
      return null;
    }
    return { id: crypto.randomUUID(), name: name.trim() || tgt, target: tgt, port: p };
  };

  const test = async () => {
    const h = draft();
    if (!h) return;
    setBusy(true);
    setStatus({ text: t.testing });
    try {
      setStatus({ text: t.testOk(await remoteTest(h)) });
    } catch (e) {
      setStatus({ text: errorText(t, toAppError(e)), error: true });
    } finally {
      setBusy(false);
    }
  };

  const add = () => {
    const h = draft();
    if (!h) return;
    setList((l) => [...l, h]);
    setName("");
    setTarget("");
    setPort("");
    setStatus(null);
  };

  return (
    <div className="backdrop">
      <div ref={ref} className="dialog wide" role="dialog" aria-modal="true" aria-labelledby="servers-title">
        <h2 id="servers-title">{t.serversTitle}</h2>
        <p className="muted small">{t.serversIntro}</p>

        {list.length > 0 && (
          <ul className="server-list">
            {list.map((h) => (
              <li key={h.id}>
                <strong>{h.name}</strong>
                <span className="mono small muted">{h.target}{h.port ? `:${h.port}` : ""}</span>
                <button className="small danger" onClick={() => setList((l) => l.filter((x) => x.id !== h.id))}>{t.remove}</button>
              </li>
            ))}
          </ul>
        )}

        <fieldset>
          <legend>{t.addServer}</legend>
          <div className="row3">
            <label className="field-col">
              <span className="label">{t.serverName}</span>
              <input type="text" value={name} maxLength={80} placeholder="web-01" onChange={(e) => setName(e.target.value)} />
            </label>
            <label className="field-col">
              <span className="label">{t.serverTarget}</span>
              <input type="text" value={target} placeholder="deploy@web-01.example.com" spellCheck={false} onChange={(e) => setTarget(e.target.value)} />
            </label>
            <label className="field-col narrow">
              <span className="label">{t.serverPort}</span>
              <input type="number" min={1} max={65535} value={port} placeholder="22" onChange={(e) => setPort(e.target.value)} />
            </label>
          </div>
          <div className="form-actions">
            <span className={`small ${status?.error ? "error-text" : "muted"}`}>{status?.text ?? t.firstConnectHint}</span>
            <span className="actions-row">
              <button onClick={test} disabled={busy || !target.trim()}>{busy ? t.testing : t.testConnection}</button>
              <button className="primary" onClick={add} disabled={!target.trim()}>{t.addServer}</button>
            </span>
          </div>
        </fieldset>

        <p className="small">{t.installHint}</p>
        <pre className="cmd">{cliInstall(version)}</pre>

        <div className="dialog-actions">
          <button onClick={onClose}>{t.cancel}</button>
          <button className="primary" onClick={() => onSave(list)}>{t.done}</button>
        </div>
      </div>
    </div>
  );
}
