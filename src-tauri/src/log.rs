//! Local diagnostics log: `Documents\Peron\Logs\peron.log` (falls back to the app log dir), rotated
//! to `peron.old.log` at 512 KB, so at most ~1 MB on disk. Only technical context is logged (no
//! port lists, command lines or paths from other processes). Users can open the folder from
//! Settings to attach it to a report.
//!
//! Why Documents: under MSIX, writes to %LOCALAPPDATA% are redirected into the package's private
//! folder, so the unredirected path can't be opened in Explorer; Documents isn't redirected and is
//! where users look. The NSIS installer creates the folder too, and its "delete application data"
//! uninstall option removes the log files (installer-hooks.nsh).

use std::io::Write;
use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};

use tauri::{AppHandle, Manager};

use crate::util::LockExt;

const MAX_BYTES: u64 = 512 * 1024;

static LOG: OnceLock<Mutex<PathBuf>> = OnceLock::new();

/// Windows: `Documents\Peron\Logs`. Linux (XDG convention) or when Documents is unavailable or not
/// writable: the app log dir.
fn log_dir(app: &AppHandle) -> Option<PathBuf> {
    #[cfg(windows)]
    let documents = app
        .path()
        .document_dir()
        .ok()
        .map(|d| d.join("Peron").join("Logs"))
        .filter(|d| std::fs::create_dir_all(d).is_ok());
    #[cfg(not(windows))]
    let documents: Option<PathBuf> = None;
    documents.or_else(|| {
        let d = app.path().app_log_dir().ok()?;
        std::fs::create_dir_all(&d).ok()?;
        Some(d)
    })
}

pub fn init(app: &AppHandle) {
    let Some(dir) = log_dir(app) else {
        return;
    };
    let _ = LOG.set(Mutex::new(dir.join("peron.log")));
    // panic = "abort" in release: the hook is the only chance to record what happened.
    let prev = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        write("PANIC", &info.to_string());
        prev(info);
    }));
    write(
        "INFO",
        &format!("Peron {} started", app.package_info().version),
    );
}

pub fn dir() -> Option<PathBuf> {
    LOG.get()
        .and_then(|p| p.lock_safe().parent().map(PathBuf::from))
}

pub fn write(level: &str, msg: &str) {
    #[cfg(debug_assertions)]
    eprintln!("[{level}] {msg}");
    let Some(lock) = LOG.get() else { return };
    let path = lock.lock_safe();
    if std::fs::metadata(&*path).is_ok_and(|m| m.len() > MAX_BYTES) {
        let _ = std::fs::rename(&*path, path.with_file_name("peron.old.log"));
    }
    if let Ok(mut f) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&*path)
    {
        let _ = writeln!(
            f,
            "{} [{level}] {msg}",
            utc_timestamp(crate::state::now_ms())
        );
    }
}

use peron_core::util::utc_timestamp;

#[macro_export]
macro_rules! log_error {
    ($($arg:tt)*) => { $crate::log::write("ERROR", &format!($($arg)*)) };
}

#[macro_export]
macro_rules! log_info {
    ($($arg:tt)*) => { $crate::log::write("INFO", &format!($($arg)*)) };
}
