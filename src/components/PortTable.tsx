import { type KeyboardEvent, useMemo, useState } from "react";
import { formatBytes, formatCpu } from "../format";
import { useT } from "../i18n";
import type { PortEntry, PortRow } from "../types";
import { FolderIcon, PowerIcon } from "./Icons";

type SortKey = "port" | "protocol" | "address" | "state" | "process" | "parent" | "age" | "cpu" | "memory";

interface Props {
  ports: PortRow[];
  now: number;
  longOpenMs: number;
  selectedId: string | null;
  onSelect: (e: PortEntry) => void;
  onKill: (e: PortEntry) => void;
  onClearFilters: () => void;
}

function sortValue(e: PortEntry, key: SortKey, now: number): number | string {
  switch (key) {
    case "port": return e.localPort;
    case "protocol": return e.protocol;
    case "address": return e.localAddr;
    case "state": return e.state;
    case "process": return e.processName.toLowerCase();
    case "parent": return (e.parentName ?? "").toLowerCase();
    case "age": return e.openedAtMs ? now - e.openedAtMs : -1;
    case "cpu": return e.cpuPercent;
    case "memory": return e.memoryBytes;
  }
}

export function isLongOpen(e: PortEntry, now: number, longOpenMs: number) {
  return e.isDev && e.state === "LISTEN" && !!e.openedAtMs && now - e.openedAtMs >= longOpenMs;
}

export function PortTable({ ports, now, longOpenMs, selectedId, onSelect, onKill, onClearFilters }: Props) {
  const t = useT();
  const [sort, setSort] = useState<{ key: SortKey; asc: boolean }>({ key: "port", asc: true });

  const columns: { key: SortKey; label: string; className?: string }[] = [
    { key: "port", label: t.colPort, className: "num" },
    { key: "protocol", label: t.colProtocol },
    { key: "address", label: t.colAddress },
    { key: "state", label: t.colState },
    { key: "process", label: t.colProcess },
    { key: "parent", label: t.colParent },
    { key: "age", label: t.colAge, className: "num" },
    { key: "cpu", label: t.colCpu, className: "num" },
    { key: "memory", label: t.colRam, className: "num" },
  ];

  const sorted = useMemo(() => {
    const dir = sort.asc ? 1 : -1;
    return [...ports].sort((a, b) => {
      const va = sortValue(a, sort.key, now);
      const vb = sortValue(b, sort.key, now);
      if (va < vb) return -dir;
      if (va > vb) return dir;
      return a.localPort - b.localPort;
    });
  }, [ports, sort, now]);

  // Keyboard: Enter/Space selects the row, ArrowUp/ArrowDown move between rows.
  const onRowKey = (ev: KeyboardEvent<HTMLTableRowElement>, e: PortRow) => {
    if (ev.target !== ev.currentTarget) return; // let the row's own buttons handle keys
    if (ev.key === "Enter" || ev.key === " ") {
      ev.preventDefault();
      onSelect(e);
    } else if (ev.key === "ArrowDown" || ev.key === "ArrowUp") {
      ev.preventDefault();
      const next = ev.key === "ArrowDown" ? ev.currentTarget.nextElementSibling : ev.currentTarget.previousElementSibling;
      (next as HTMLElement | null)?.focus();
    }
  };

  const toggleSort = (key: SortKey) =>
    setSort((s) => (s.key === key ? { key, asc: !s.asc } : { key, asc: !["age", "cpu", "memory"].includes(key) }));

  if (ports.length === 0) {
    return (
      <div className="empty">
        <p className="empty-title">{t.emptyFiltered}</p>
        <p className="muted">{t.emptyHint}</p>
        <button onClick={onClearFilters}>{t.clearFilters}</button>
      </div>
    );
  }

  return (
    <div className="table-wrap">
      <table className="ports">
        <thead>
          <tr>
            {columns.map((c) => (
              <th
                key={c.key}
                className={c.className}
                aria-sort={sort.key === c.key ? (sort.asc ? "ascending" : "descending") : "none"}
              >
                <button className="th-sort" onClick={() => toggleSort(c.key)}>
                  {c.label}
                  <span className="sort" aria-hidden="true">{sort.key === c.key ? (sort.asc ? "▲" : "▼") : ""}</span>
                </button>
              </th>
            ))}
            <th aria-label={t.close} />
          </tr>
        </thead>
        <tbody>
          {sorted.map((e) => {
            const project = e.project;
            const long = isLongOpen(e, now, longOpenMs);
            return (
              <tr
                key={e.id}
                className={e.id === selectedId ? "selected" : ""}
                tabIndex={0}
                aria-selected={e.id === selectedId}
                onClick={() => onSelect(e)}
                onKeyDown={(ev) => onRowKey(ev, e)}
              >
                <td className="num port">{e.localPort}</td>
                <td>
                  <span className={`badge ${e.protocol.toLowerCase()}`}>{e.protocol}</span>
                  <span className="muted small"> {e.ipVersions.map((v) => `v${v}`).join("+")}</span>
                </td>
                <td>
                  <span className="mono">{e.addrs.join(", ")}</span>
                  {e.remoteAddr ? (
                    <div className="muted small mono">→ {e.remoteAddr}:{e.remotePort}</div>
                  ) : e.isLocalhostOnly ? (
                    <span className="badge local" title={t.badgeLocalTitle}>{t.badgeLocal}</span>
                  ) : (
                    e.state !== "BOUND" && (
                      <span className="badge exposed" title={t.badgeExposedTitle}>{t.badgeExposed}</span>
                    )
                  )}
                </td>
                <td><span className={`state s-${e.state.toLowerCase()}`}>{e.state}</span></td>
                <td>
                  <div className="proc">
                    {e.processName}
                    {e.isDev && <span className="badge dev">{t.badgeDev}</span>}
                    {e.isSystem && <span className="badge sys">{t.badgeSystem}</span>}
                  </div>
                  <div className="muted small sub">
                    PID {e.pid}
                    {project && (
                      <span className="project" title={e.cwd ?? undefined}>
                        <FolderIcon size={12} /> {project}
                      </span>
                    )}
                  </div>
                </td>
                <td>{e.parentName ?? <span className="muted">—</span>}</td>
                <td
                  className={`num ${long ? "long" : ""}`}
                  title={long ? t.longOpenTitle : e.openedAtSource === "process" ? t.fallbackStartTitle : undefined}
                >
                  {e.openedAtMs ? t.duration(now - e.openedAtMs) : "—"}
                  {e.openedAtSource === "process" && e.openedAtMs ? <span className="muted">*</span> : null}
                </td>
                <td className="num">{e.accessible || e.cpuPercent > 0 ? formatCpu(e.cpuPercent) : "—"}</td>
                <td className="num">{e.memoryBytes > 0 ? formatBytes(e.memoryBytes) : "—"}</td>
                <td className="actions">
                  <button
                    className="danger small"
                    disabled={e.isProtected}
                    title={e.isProtected ? t.protectedTitle : t.closeTitle}
                    onClick={(ev) => {
                      ev.stopPropagation();
                      onKill(e);
                    }}
                  >
                    <PowerIcon size={13} /> {t.close}
                  </button>
                </td>
              </tr>
            );
          })}
        </tbody>
      </table>
    </div>
  );
}
