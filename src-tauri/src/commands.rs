//! The only UI ↔ Rust surface.

use serde::Serialize;
use tauri::{AppHandle, Manager};

use crate::error::{AppError, AppResult};
use crate::i18n::Lang;
use crate::model::PortEntry;
use crate::monitor;
use crate::proc::info::{self, Detail, ProcRef};
use crate::proc::{elevate, kill};
use crate::settings::{self, Settings};
use crate::state::{AppState, ModeInfo};
use crate::tray;
use crate::util::LockExt;
use crate::{feedback, platform};

async fn blocking<T: Send + 'static>(
    f: impl FnOnce() -> AppResult<T> + Send + 'static,
) -> AppResult<T> {
    tauri::async_runtime::spawn_blocking(f)
        .await
        .map_err(|e| AppError::Other(e.to_string()))?
}

#[tauri::command]
pub async fn list_ports(app: AppHandle) -> AppResult<Vec<PortEntry>> {
    blocking(move || monitor::rescan(&app, Detail::Full)).await
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProcessDetails {
    pub pid: u32,
    pub parent_chain: Vec<ProcRef>,
    pub children: Vec<ProcRef>,
}

#[tauri::command]
pub async fn get_process_details(app: AppHandle, pid: u32) -> AppResult<ProcessDetails> {
    blocking(move || {
        let state = app.state::<AppState>();
        let mut sys = state.sys.lock_safe();
        // Process trees need every process; done only on demand.
        info::refresh_all_light(&mut sys);
        let children = info::descendants(&sys, pid)
            .into_iter()
            .map(|(pid, name)| ProcRef {
                pid,
                name,
                exe: None,
            })
            .collect();
        Ok(ProcessDetails {
            pid,
            parent_chain: info::parent_chain(&sys, pid),
            children,
        })
    })
    .await
}

/// Called only from the UI's confirmation dialog.
#[tauri::command]
pub async fn kill_process(
    app: AppHandle,
    pid: u32,
    expected_start_ms: i64,
    tree: bool,
) -> AppResult<kill::KillReport> {
    blocking(move || {
        let report = {
            let state = app.state::<AppState>();
            let mut sys = state.sys.lock_safe();
            kill::kill(&mut sys, pid, expected_start_ms, tree)?
        };
        // Give the OS a moment to release the sockets before rescanning.
        std::thread::sleep(std::time::Duration::from_millis(300));
        let _ = monitor::rescan(&app, Detail::Full);
        Ok(report)
    })
    .await
}

#[tauri::command]
pub fn get_settings(app: AppHandle) -> Settings {
    app.state::<AppState>().settings.lock_safe().clone()
}

#[tauri::command]
pub async fn save_settings(app: AppHandle, settings: Settings) -> AppResult<Settings> {
    // Async: toggling the MSIX StartupTask waits on WinRT; keep that off the main thread.
    settings::apply(&app, settings)
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppInfo {
    /// Effective language (user choice → installer → OS → English).
    pub language: Lang,
    pub elevated: bool,
    pub can_elevate: bool,
    /// "store" (MSIX, updated by the Store) or "direct" (NSIS/Linux packages, checks GitHub).
    pub channel: &'static str,
    pub platform: &'static str,
    pub os: String,
    pub version: String,
    pub mode: ModeInfo,
}

#[tauri::command]
pub fn get_app_info(app: AppHandle) -> AppInfo {
    let state = app.state::<AppState>();
    let language = state.settings.lock_safe().lang();
    let mode = *state.mode.lock_safe();
    AppInfo {
        language,
        elevated: elevate::is_elevated(),
        can_elevate: elevate::can_elevate(),
        channel: if platform::is_packaged() {
            "store"
        } else {
            "direct"
        },
        platform: std::env::consts::OS,
        os: platform::os_description(),
        version: app.package_info().version.to_string(),
        mode,
    }
}

/// Reserves a feedback send slot (client-side rate limit; see feedback.rs).
#[tauri::command]
pub fn feedback_reserve(app: AppHandle) -> AppResult<()> {
    feedback::reserve(&app)
}

/// The UI shows its (hidden-created) window after the first render; re-evaluate the scan pace.
#[tauri::command]
pub fn window_shown(app: AppHandle) {
    app.state::<AppState>().wake_monitor();
}

/// Hides to the tray, releasing the WebView.
#[tauri::command]
pub fn hide_to_tray(app: AppHandle) {
    tray::hide_main(&app);
}

#[tauri::command]
pub fn relaunch_as_admin(app: AppHandle) -> AppResult<()> {
    if !elevate::can_elevate() {
        return Err(AppError::Other(
            "elevation is not available in this build".into(),
        ));
    }
    elevate::relaunch_as_admin()?;
    app.exit(0);
    Ok(())
}

/// Opens the diagnostics log folder (Settings → "Open log folder").
#[tauri::command]
pub fn open_log_folder(app: AppHandle) -> AppResult<()> {
    use tauri_plugin_opener::OpenerExt;
    let dir = crate::log::dir().ok_or_else(|| AppError::Other("log folder unavailable".into()))?;
    app.opener()
        .open_path(dir.to_string_lossy(), None::<&str>)
        .map_err(|e| AppError::Other(e.to_string()))
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportResult {
    pub file_name: String,
    /// Saved file (when `save`), for "show in folder".
    pub path: Option<String>,
    /// Rendered text (when not saving), for the clipboard.
    pub text: Option<String>,
}

/// Renders the given entries (the rows the user sees) as TXT/CSV/JSON and either saves the file
/// to the Downloads folder with a timestamped name or returns the text for the clipboard.
/// `host` = remote host id whose last snapshot to export; `None` = this computer.
#[tauri::command]
pub async fn export_snapshot(
    app: AppHandle,
    format: peron_core::export::Format,
    ids: Vec<String>,
    host: Option<String>,
    save: bool,
) -> AppResult<ExportResult> {
    use peron_core::export::{self, Snapshot};
    blocking(move || {
        let state = app.state::<AppState>();
        let lang = state.settings.lock_safe().lang();
        let base: Snapshot = match host {
            Some(id) => state
                .remote
                .lock_safe()
                .get(&id)
                .cloned()
                .ok_or_else(|| AppError::Other("remote snapshot not loaded".into()))?,
            None => Snapshot::new(
                "peron",
                &app.package_info().version.to_string(),
                elevate::is_elevated(),
                state.snapshot.lock_safe().clone(),
                crate::state::now_ms(),
            ),
        };
        let wanted: std::collections::HashSet<&str> = ids.iter().map(String::as_str).collect();
        let snapshot = Snapshot {
            entries: base
                .entries
                .iter()
                .filter(|e| wanted.contains(e.id.as_str()))
                .cloned()
                .collect(),
            generated_at_ms: crate::state::now_ms(),
            ..base.clone()
        };
        let text = export::render(&snapshot, format, lang);
        let file_name =
            export::file_name("peron", &snapshot.host, snapshot.generated_at_ms, format);
        if !save {
            return Ok(ExportResult {
                file_name,
                path: None,
                text: Some(text),
            });
        }
        let dir = app
            .path()
            .download_dir()
            .or_else(|_| app.path().home_dir())
            .map_err(|e| AppError::Other(e.to_string()))?;
        let path = dir.join(&file_name);
        crate::util::write_atomic(&path, text.as_bytes())
            .map_err(|e| AppError::Other(e.to_string()))?;
        Ok(ExportResult {
            file_name,
            path: Some(path.to_string_lossy().into_owned()),
            text: None,
        })
    })
    .await
}

// --- History ------------------------------------------------------------------------------------

/// Port open/close events, newest first.
#[tauri::command]
pub fn get_history(app: AppHandle) -> Vec<peron_core::history::PortEvent> {
    let mut events = app.state::<AppState>().history.lock_safe().clone();
    events.reverse();
    events
}

#[tauri::command]
pub fn clear_history(app: AppHandle) {
    app.state::<AppState>().history.lock_safe().clear();
    crate::history_store::clear(&app);
}

/// Same save/clipboard behavior as `export_snapshot`, for the History tab.
#[tauri::command]
pub async fn export_history(
    app: AppHandle,
    format: peron_core::export::Format,
    save: bool,
) -> AppResult<ExportResult> {
    use peron_core::{export, history};
    blocking(move || {
        let state = app.state::<AppState>();
        let lang = state.settings.lock_safe().lang();
        let mut events = state.history.lock_safe().clone();
        events.reverse();
        let now = crate::state::now_ms();
        let host = export::host_name();
        let text = history::render(&events, format, lang, &host, now);
        let file_name = export::file_name("peron-history", &host, now, format);
        if !save {
            return Ok(ExportResult {
                file_name,
                path: None,
                text: Some(text),
            });
        }
        let dir = app
            .path()
            .download_dir()
            .or_else(|_| app.path().home_dir())
            .map_err(|e| AppError::Other(e.to_string()))?;
        let path = dir.join(&file_name);
        crate::util::write_atomic(&path, text.as_bytes())
            .map_err(|e| AppError::Other(e.to_string()))?;
        Ok(ExportResult {
            file_name,
            path: Some(path.to_string_lossy().into_owned()),
            text: None,
        })
    })
    .await
}

// --- Remote (SSH) -------------------------------------------------------------------------------

fn remote_host(app: &AppHandle, id: &str) -> AppResult<peron_core::remote::RemoteHost> {
    app.state::<AppState>()
        .settings
        .lock_safe()
        .remote_hosts
        .iter()
        .find(|h| h.id == id)
        .cloned()
        .ok_or_else(|| AppError::Other(format!("unknown host {id}")))
}

/// Fetches a server's ports through `ssh <target> peron-cli list --json` and caches the result
/// (export and the kill guard use the cached snapshot).
#[tauri::command]
pub async fn remote_list(app: AppHandle, id: String) -> AppResult<peron_core::export::Snapshot> {
    blocking(move || {
        let host = remote_host(&app, &id)?;
        let snapshot = peron_core::remote::fetch(&host)?;
        app.state::<AppState>()
            .remote
            .lock_safe()
            .insert(id, snapshot.clone());
        Ok(snapshot)
    })
    .await
}

/// "Test connection" in the server dialog, before the host is saved.
#[tauri::command]
pub async fn remote_test(host: peron_core::remote::RemoteHost) -> AppResult<String> {
    blocking(move || {
        let s = peron_core::remote::fetch(&host)?;
        Ok(format!("{} · {} · peron-cli {}", s.host, s.os, s.version))
    })
    .await
}

/// Ends a process on a server. Called only from the confirmation dialog;
/// the PID must be in the last snapshot shown to the user, and the start time guards PID reuse.
#[tauri::command]
pub async fn remote_kill(
    app: AppHandle,
    id: String,
    pid: u32,
    expected_start_ms: i64,
    tree: bool,
) -> AppResult<kill::KillReport> {
    blocking(move || {
        let host = remote_host(&app, &id)?;
        let known = app
            .state::<AppState>()
            .remote
            .lock_safe()
            .get(&id)
            .is_some_and(|s| {
                s.entries
                    .iter()
                    .any(|e| e.pid == pid && e.process_start_ms == expected_start_ms)
            });
        if !known {
            return Err(AppError::Changed);
        }
        peron_core::remote::kill(&host, pid, expected_start_ms, tree)?;
        Ok(kill::KillReport {
            killed: vec![pid],
            failed: Vec::new(),
        })
    })
    .await
}
