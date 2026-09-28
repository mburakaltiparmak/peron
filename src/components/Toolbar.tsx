import { forwardRef } from "react";
import { useT } from "../i18n";
import type { ExportFormat, Settings, ViewMode } from "../types";
import { ExportMenu } from "./ExportMenu";
import { LOCAL } from "./FilterBar";
import { RefreshIcon, SearchIcon, SettingsIcon, ShieldIcon } from "./Icons";

export type Tab = "ports" | "history" | "contact";

interface Props {
  tab: Tab;
  onTab: (tab: Tab) => void;
  /** Pre-release label ("BETA") for versions like 0.1.0-beta.1; null for stable releases. */
  prerelease: string | null;
  settings: Settings;
  search: string;
  loading: boolean;
  /** Show the "relaunch as admin" button (Windows, not yet elevated). */
  showAdmin: boolean;
  /** Selected machine (FilterBar); the admin button only applies to this computer. */
  host: string;
  onSearch: (q: string) => void;
  onChange: (patch: Partial<Settings>) => void;
  onRefresh: () => void;
  onExport: (format: ExportFormat, save: boolean) => void;
  onOpenSettings: () => void;
  onRelaunchAdmin: () => void;
}

export const Toolbar = forwardRef<HTMLInputElement, Props>(function Toolbar(p, searchRef) {
  const t = useT();
  const views: { value: ViewMode; label: string }[] = [
    { value: "listen", label: t.viewListen },
    { value: "all", label: t.viewAll },
  ];
  const tabs: [Tab, string][] = [
    ["ports", t.tabPorts],
    ["history", t.tabHistory],
    ["contact", t.tabContact],
  ];
  return (
    <header className="toolbar">
      <div className="brand">
        <img src="/logo.png" alt="" width={24} height={24} />
        {p.prerelease && <span className="beta-badge">{p.prerelease}</span>}
      </div>

      <nav className="tabs" role="tablist">
        {tabs.map(([value, label]) => (
          <button key={value} role="tab" aria-selected={p.tab === value} className={p.tab === value ? "active" : ""} onClick={() => p.onTab(value)}>
            {label}
          </button>
        ))}
      </nav>

      {p.tab === "ports" && (
        <>
          <div className="segmented">
            {views.map((v) => (
              <button
                key={v.value}
                aria-pressed={p.settings.viewMode === v.value}
                className={p.settings.viewMode === v.value ? "active" : ""}
                onClick={() => p.onChange({ viewMode: v.value })}
              >
                {v.label}
              </button>
            ))}
          </div>

          <label className="search">
            <SearchIcon size={15} />
            <input
              ref={searchRef}
              type="search"
              aria-label={t.searchLabel}
              placeholder={t.searchPlaceholder}
              value={p.search}
              onChange={(e) => p.onSearch(e.target.value)}
            />
          </label>
        </>
      )}

      <div className="spacer" />

      {p.showAdmin && p.host === LOCAL && (
        <button className="ghost" onClick={p.onRelaunchAdmin} title={t.adminTitle}>
          <ShieldIcon size={15} /> {t.admin}
        </button>
      )}
      {p.tab === "ports" && (
        <>
          <ExportMenu onExport={p.onExport} />
          <button onClick={p.onRefresh} disabled={p.loading} title={`${t.refresh} (F5)`}>
            <span className={p.loading ? "spin" : ""}>
              <RefreshIcon size={15} />
            </span>
            {p.loading ? t.refreshing : t.refresh}
          </button>
        </>
      )}
      <button className="icon" onClick={p.onOpenSettings} title={t.settings} aria-label={t.settings}>
        <SettingsIcon size={16} />
      </button>
    </header>
  );
});
