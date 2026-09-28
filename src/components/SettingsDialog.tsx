import { useCallback, useEffect, useRef, useState } from "react";
import { useDialogFocus } from "../hooks/useDialogFocus";
import { openLogFolder, toAppError } from "../api";
import { errorText, useT } from "../i18n";
import { applyTheme } from "../theme";
import type { AppInfo, Lang, Settings, Theme } from "../types";

/** Same limits as `Settings::sanitized` (src-tauri/src/settings.rs): shown and applied in the form
 *  so a value is never changed silently on save. */
const LIMITS = { threshold: [1, 7 * 24 * 60], repeat: [5, 24 * 60] } as const;
const clamp = (n: number, [min, max]: readonly [number, number]) =>
  Number.isFinite(n) ? Math.min(max, Math.max(min, Math.round(n))) : min;

interface Props {
  settings: Settings;
  /** Effective language, shown when the user never picked one. */
  language: Lang;
  channel: AppInfo["channel"];
  onSave: (s: Settings) => Promise<void>;
  onCancel: () => void;
}

export function SettingsDialog({ settings, language, channel, onSave, onCancel }: Props) {
  const t = useT();
  const [draft, setDraft] = useState<Settings>({ ...settings, language: settings.language ?? language });
  const [error, setError] = useState<string | null>(null);
  const dialogRef = useRef<HTMLDivElement>(null);
  useDialogFocus(dialogRef);
  const set = (patch: Partial<Settings>) => setDraft((d) => ({ ...d, ...patch }));

  // Live preview: the theme changes as soon as it is picked; Cancel/Esc puts the saved one back.
  useEffect(() => applyTheme(draft.theme), [draft.theme]);
  const savedTheme = settings.theme;
  const cancel = useCallback(() => {
    applyTheme(savedTheme);
    onCancel();
  }, [savedTheme, onCancel]);

  useEffect(() => {
    const onKey = (ev: KeyboardEvent) => {
      if (ev.key === "Escape") {
        ev.stopPropagation();
        cancel();
      }
    };
    window.addEventListener("keydown", onKey, true);
    return () => window.removeEventListener("keydown", onKey, true);
  }, [cancel]);

  const save = async () => {
    try {
      await onSave({
        ...draft,
        reminderThresholdMinutes: clamp(draft.reminderThresholdMinutes, LIMITS.threshold),
        reminderRepeatMinutes: clamp(draft.reminderRepeatMinutes, LIMITS.repeat),
      });
    } catch (e) {
      setError(errorText(t, toAppError(e)));
    }
  };

  const themes: { value: Theme; label: string }[] = [
    { value: "system", label: t.themeSystem },
    { value: "light", label: t.themeLight },
    { value: "dark", label: t.themeDark },
  ];

  return (
    <div className="backdrop" onClick={cancel}>
      <div ref={dialogRef} className="dialog wide" role="dialog" aria-modal="true" aria-labelledby="settings-title" onClick={(e) => e.stopPropagation()}>
        <h2 id="settings-title">{t.settingsTitle}</h2>

        <fieldset>
          <legend>{t.secGeneral}</legend>
          <label className="field">
            {t.language}
            <select value={draft.language ?? language} onChange={(e) => set({ language: e.target.value as Lang })}>
              <option value="tr">Türkçe</option>
              <option value="en">English</option>
            </select>
          </label>
          <div className="field">
            {t.theme}
            <div className="segmented small">
              {themes.map((th) => (
                <button
                  key={th.value}
                  className={draft.theme === th.value ? "active" : ""}
                  aria-pressed={draft.theme === th.value}
                  onClick={() => set({ theme: th.value })}
                >
                  {th.label}
                </button>
              ))}
            </div>
          </div>
        </fieldset>

        <fieldset>
          <legend>{t.secPerformance}</legend>
          <label className="check">
            <input type="checkbox" checked={draft.autoRefresh} onChange={(e) => set({ autoRefresh: e.target.checked })} />
            {t.autoRefresh}
          </label>
          <label className="field">
            {t.refreshInterval}
            <select
              value={draft.refreshIntervalSecs}
              disabled={!draft.autoRefresh}
              onChange={(e) => set({ refreshIntervalSecs: Number(e.target.value) })}
            >
              {[2, 3, 5, 10, 30].map((s) => (
                <option key={s} value={s}>{t.seconds(s)}</option>
              ))}
            </select>
          </label>
          <p className="muted small">{t.perfNote}</p>
        </fieldset>

        <fieldset>
          <legend>{t.secReminders}</legend>
          <label className="check">
            <input type="checkbox" checked={draft.reminderEnabled} onChange={(e) => set({ reminderEnabled: e.target.checked })} />
            {t.reminderEnabled}
          </label>
          <label className="check">
            <input type="checkbox" checked={draft.alertExposed} onChange={(e) => set({ alertExposed: e.target.checked })} />
            {t.alertExposed}
          </label>
          <label className="field">
            {t.reminderThreshold}
            <input
              type="number"
              min={LIMITS.threshold[0]}
              max={LIMITS.threshold[1]}
              value={draft.reminderThresholdMinutes}
              disabled={!draft.reminderEnabled}
              onChange={(e) => set({ reminderThresholdMinutes: Number(e.target.value) })}
              onBlur={() => set({ reminderThresholdMinutes: clamp(draft.reminderThresholdMinutes, LIMITS.threshold) })}
            />
          </label>
          <label className="field">
            {t.reminderRepeat}
            <input
              type="number"
              min={LIMITS.repeat[0]}
              max={LIMITS.repeat[1]}
              value={draft.reminderRepeatMinutes}
              disabled={!draft.reminderEnabled}
              onChange={(e) => set({ reminderRepeatMinutes: Number(e.target.value) })}
              onBlur={() => set({ reminderRepeatMinutes: clamp(draft.reminderRepeatMinutes, LIMITS.repeat) })}
            />
          </label>
        </fieldset>

        <fieldset>
          <legend>{t.secStartup}</legend>
          <label className="check">
            <input type="checkbox" checked={draft.closeToTray} onChange={(e) => set({ closeToTray: e.target.checked })} />
            {t.closeToTray}
          </label>
          <label className="check">
            <input type="checkbox" checked={draft.autostart} onChange={(e) => set({ autostart: e.target.checked })} />
            {t.autostart}
          </label>
          {channel === "direct" && (
            <label className="check">
              <input type="checkbox" checked={draft.checkUpdates} onChange={(e) => set({ checkUpdates: e.target.checked })} />
              {t.checkUpdates}
            </label>
          )}
        </fieldset>

        <fieldset>
          <legend>{t.secDiagnostics}</legend>
          <div className="field">
            <span className="muted small">{t.diagnosticsNote}</span>
            <button onClick={() => openLogFolder().catch((e) => setError(errorText(t, toAppError(e))))}>
              {t.openLogFolder}
            </button>
          </div>
        </fieldset>

        <p className="muted small">{t.shortcuts}</p>

        {error && <div className="error">{error}</div>}

        <div className="dialog-actions">
          <button onClick={cancel}>{t.cancel}</button>
          <button className="primary" onClick={save}>{t.save}</button>
        </div>
      </div>
    </div>
  );
}
