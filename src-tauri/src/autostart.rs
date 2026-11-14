//! Start with the OS, per distribution channel:
//! - MSIX (Microsoft Store): the package's `StartupTask` (HKCU\Run writes are virtualized there).
//! - NSIS / Linux / macOS: tauri-plugin-autostart (Run key / XDG autostart entry / LaunchAgent)
//!   with `--minimized`.

use tauri::AppHandle;
use tauri_plugin_autostart::ManagerExt;

use crate::error::{AppError, AppResult};

pub fn is_enabled(app: &AppHandle) -> bool {
    #[cfg(windows)]
    if crate::platform::is_packaged() {
        return crate::platform::startup_task_enabled();
    }
    app.autolaunch().is_enabled().unwrap_or(false)
}

pub fn set(app: &AppHandle, enable: bool) -> AppResult<()> {
    #[cfg(windows)]
    if crate::platform::is_packaged() {
        return crate::platform::set_startup_task(enable)
            .map_err(|e| AppError::Other(format!("autostart: {e}")));
    }
    let autolaunch = app.autolaunch();
    let result = if enable {
        autolaunch.enable()
    } else {
        autolaunch.disable()
    };
    result.map_err(|e| AppError::Other(format!("autostart: {e}")))
}

/// Whether this launch came from autostart (then Peron starts tray-only, without a window).
pub fn launched_at_login(args: &[String], minimized_arg: &str) -> bool {
    let by_arg = args.iter().any(|a| a == minimized_arg);
    #[cfg(windows)]
    if !by_arg && crate::platform::is_packaged() {
        return crate::platform::launched_by_startup_task();
    }
    by_arg
}
