//! User settings persisted as JSON in the app config dir (%APPDATA%\com.peron.app\settings.json).

use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, Manager};

use crate::error::{AppError, AppResult};
use crate::i18n::Lang;
use crate::state::AppState;
use crate::tray;
use crate::util::{self, LockExt};
pub use peron_core::remote::RemoteHost;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ViewMode {
    Listen,
    All,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Theme {
    System,
    Light,
    Dark,
}

impl Theme {
    /// Native window theme (title bar); `None` follows the OS. The UI applies later changes itself.
    pub fn window_theme(self) -> Option<tauri::Theme> {
        match self {
            Theme::System => None,
            Theme::Light => Some(tauri::Theme::Light),
            Theme::Dark => Some(tauri::Theme::Dark),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Settings {
    /// `None` = not chosen in the app yet; resolved via `Lang::resolve`.
    pub language: Option<Lang>,
    pub theme: Theme,
    pub view_mode: ViewMode,
    pub hide_system: bool,
    pub show_udp: bool,
    pub auto_refresh: bool,
    pub refresh_interval_secs: u32,
    pub reminder_enabled: bool,
    pub reminder_threshold_minutes: u32,
    pub reminder_repeat_minutes: u32,
    pub close_to_tray: bool,
    pub autostart: bool,
    /// Direct-download builds only: look for a newer release on GitHub (at most once a day).
    /// Store builds are updated by the Store and never check.
    pub check_updates: bool,
    /// Notify when a (non-system) process starts listening on a network-reachable address.
    pub alert_exposed: bool,
    /// SSH hosts for the remote view (see peron_core::remote).
    pub remote_hosts: Vec<RemoteHost>,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            language: None,
            theme: Theme::System,
            view_mode: ViewMode::Listen,
            hide_system: true,
            show_udp: false,
            auto_refresh: true,
            refresh_interval_secs: 3,
            reminder_enabled: true,
            reminder_threshold_minutes: 240,
            reminder_repeat_minutes: 60,
            close_to_tray: true,
            autostart: false,
            check_updates: true,
            alert_exposed: true,
            remote_hosts: Vec::new(),
        }
    }
}

impl Settings {
    pub fn lang(&self) -> Lang {
        Lang::resolve(self.language)
    }

    /// Clamps values the UI could send out of range.
    pub fn sanitized(mut self) -> Self {
        self.refresh_interval_secs = self.refresh_interval_secs.clamp(1, 60);
        self.reminder_threshold_minutes = self.reminder_threshold_minutes.clamp(1, 7 * 24 * 60);
        self.reminder_repeat_minutes = self.reminder_repeat_minutes.clamp(5, 24 * 60);
        // Never keep a host whose target could be read by ssh as an option (see remote.rs).
        self.remote_hosts.retain(|h| h.is_valid());
        self
    }
}

fn path(app: &AppHandle) -> Option<PathBuf> {
    app.path()
        .app_config_dir()
        .ok()
        .map(|d| d.join("settings.json"))
}

pub fn load(app: &AppHandle) -> Settings {
    path(app)
        .and_then(|p| util::read_with_backup(&p, |s| serde_json::from_str::<Settings>(s).ok()))
        .unwrap_or_default()
        .sanitized()
}

pub const SETTINGS_CHANGED: &str = "settings-changed";

/// Validates, persists and applies settings (autostart, tray language, monitor pace), then tells
/// the UI. Used by the settings dialog and the tray language menu.
pub fn apply(app: &AppHandle, settings: Settings) -> AppResult<Settings> {
    let settings = settings.sanitized();
    if settings.autostart != crate::autostart::is_enabled(app) {
        crate::autostart::set(app, settings.autostart)?;
    }
    save(app, &settings).map_err(|e| AppError::Other(format!("settings: {e}")))?;
    let state = app.state::<AppState>();
    *state.settings.lock_safe() = settings.clone();
    tray::refresh(app);
    state.wake_monitor();
    let _ = app.emit(SETTINGS_CHANGED, &settings);
    Ok(settings)
}

pub fn save(app: &AppHandle, settings: &Settings) -> std::io::Result<()> {
    let Some(p) = path(app) else { return Ok(()) };
    util::write_atomic(&p, serde_json::to_string_pretty(settings)?.as_bytes())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn partial_json_uses_defaults() {
        let s: Settings =
            serde_json::from_str(r#"{"viewMode":"all","refreshIntervalSecs":0}"#).unwrap();
        let s = s.sanitized();
        assert_eq!(s.view_mode, ViewMode::All);
        assert_eq!(s.refresh_interval_secs, 1);
        assert!(s.hide_system);
    }
}
