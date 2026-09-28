import { useT } from "../i18n";
import type { RemoteHost, Settings } from "../types";
import { ServerIcon } from "./Icons";

export type QuickFilter = "all" | "dev" | "long";

/** Value of the machine selector: "local" or a remote host id. */
export const LOCAL = "local";
const MANAGE = "__manage__";

interface Props {
  settings: Settings;
  filter: QuickFilter;
  counts: Record<QuickFilter, number>;
  onFilter: (f: QuickFilter) => void;
  onChange: (patch: Partial<Settings>) => void;
  host: string;
  onHost: (id: string) => void;
  onManageHosts: () => void;
}

export function FilterBar({ settings, filter, counts, onFilter, onChange, host, onHost, onManageHosts }: Props) {
  const t = useT();
  const chips: { value: QuickFilter; label: string; title?: string }[] = [
    { value: "all", label: t.filterAll },
    { value: "dev", label: t.filterDev },
    { value: "long", label: t.filterLong, title: t.filterLongTitle(settings.reminderThresholdMinutes) },
  ];
  const hosts: RemoteHost[] = settings.remoteHosts;
  return (
    <div className="filterbar">
      <label className="host-select" title={t.machineTitle}>
        <ServerIcon size={15} />
        <select
          aria-label={t.machine}
          value={host}
          onChange={(e) => (e.target.value === MANAGE ? onManageHosts() : onHost(e.target.value))}
        >
          <option value={LOCAL}>{t.thisComputer}</option>
          {hosts.map((h) => (
            <option key={h.id} value={h.id}>{h.name}</option>
          ))}
          <option value={MANAGE}>{t.manageServers}</option>
        </select>
      </label>
      <div className="chips" role="radiogroup">
        {chips.map((c) => (
          <button
            key={c.value}
            role="radio"
            aria-checked={filter === c.value}
            className={`chip ${filter === c.value ? "active" : ""} ${c.value === "long" && counts.long > 0 ? "warn" : ""}`}
            title={c.title}
            onClick={() => onFilter(c.value)}
          >
            {c.label}
            <span className="count">{counts[c.value]}</span>
          </button>
        ))}
      </div>
      <div className="spacer" />
      <label className="check">
        <input type="checkbox" checked={settings.hideSystem} onChange={(e) => onChange({ hideSystem: e.target.checked })} />
        {t.hideSystem}
      </label>
      <label className="check">
        <input type="checkbox" checked={settings.showUdp} onChange={(e) => onChange({ showUdp: e.target.checked })} />
        {t.showUdp}
      </label>
    </div>
  );
}
