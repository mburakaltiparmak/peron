//! Desktop notifications under Peron's own identity; clicking one opens the main window.
//!
//! - Windows: unpackaged apps must register their AppUserModelID, otherwise Windows shows the
//!   toast as "Windows PowerShell" (what tauri-plugin-notification falls back to) or drops it. We
//!   register `HKCU\Software\Classes\AppUserModelId\<id>` with a display name and icon.
//! - Linux: freedesktop notifications via `notify-rust` (D-Bus), app name "Peron".

use std::path::PathBuf;

use tauri::{AppHandle, Manager};

use crate::tray;

const ICON_PNG: &[u8] = include_bytes!("../icons/128x128.png");

fn write_icon(app: &AppHandle) -> Option<PathBuf> {
    let dir = app.path().app_local_data_dir().ok()?;
    std::fs::create_dir_all(&dir).ok()?;
    let path = dir.join("notification-icon.png");
    if std::fs::read(&path).ok().as_deref() != Some(ICON_PNG) {
        std::fs::write(&path, ICON_PNG).ok()?;
    }
    Some(path)
}

fn open_main(app: &AppHandle) {
    let h = app.clone();
    // Notification callbacks run on foreign threads; window work belongs on the main thread.
    let _ = app.run_on_main_thread(move || tray::show_main(&h));
}

/// Registers the notification identity (idempotent). Failures only degrade notifications.
#[cfg(windows)]
pub fn register(app: &AppHandle) {
    // MSIX: the package already provides an identity (name + logo from the manifest).
    if crate::platform::is_packaged() {
        return;
    }
    let key = format!(
        r"Software\Classes\AppUserModelId\{}",
        app.config().identifier
    );
    let icon = write_icon(app).map(|p| p.to_string_lossy().into_owned());
    let mut values = vec![("DisplayName", "Peron")];
    if let Some(icon) = icon.as_deref() {
        values.push(("IconUri", icon));
    }
    if let Err(e) = crate::platform::hkcu_set_strings(&key, &values) {
        crate::log_error!("AUMID registration failed: {e}");
    }
}

#[cfg(target_os = "linux")]
pub fn register(app: &AppHandle) {
    // Nothing to register; just make sure the icon file exists for notifications.
    let _ = write_icon(app);
}

#[cfg(windows)]
pub fn show(app: &AppHandle, title: &str, body: &str) {
    let handle = app.clone();
    // Packaged (Store) → the package AUMID; unpackaged → our registered identifier.
    let aumid = crate::platform::package_aumid().unwrap_or_else(|| app.config().identifier.clone());
    let result = tauri_winrt_notification::Toast::new(&aumid)
        .title(title)
        .text1(body)
        .on_activated(move |_| {
            open_main(&handle);
            Ok(())
        })
        .show();
    if let Err(e) = result {
        crate::log_error!("notification failed: {e}");
    }
}

#[cfg(target_os = "linux")]
pub fn show(app: &AppHandle, title: &str, body: &str) {
    let mut n = notify_rust::Notification::new();
    n.appname("Peron")
        .summary(title)
        .body(body)
        .action("default", "Peron");
    if let Some(icon) = write_icon(app) {
        n.icon(&icon.to_string_lossy());
    }
    match n.show() {
        Ok(handle) => {
            let app = app.clone();
            // wait_for_action blocks until the notification is clicked or closed.
            std::thread::spawn(move || {
                handle.wait_for_action(|action| {
                    if action == "default" {
                        open_main(&app);
                    }
                })
            });
        }
        Err(e) => crate::log_error!("notification failed: {e}"),
    }
}
