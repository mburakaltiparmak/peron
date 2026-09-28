//! Linux helpers mirroring `platform/windows.rs`.

/// Linux packages (deb/rpm/AppImage) can't ask questions during install, so the language comes
/// from the locale (`LANG`) until the user changes it in Settings.
pub fn installer_language() -> Option<String> {
    None
}

/// Idle detection has no portable API across X11/Wayland; report "active" so the Idle mode
/// simply never triggers (Background mode is already frugal).
pub fn idle_ms() -> u64 {
    0
}

/// True when the locale (LC_ALL → LC_MESSAGES → LANG) is Turkish, e.g. `tr_TR.UTF-8`.
pub fn ui_language_is_turkish() -> bool {
    ["LC_ALL", "LC_MESSAGES", "LANG"]
        .iter()
        .filter_map(|k| std::env::var(k).ok())
        .find(|v| !v.is_empty())
        .is_some_and(|v| v.to_ascii_lowercase().starts_with("tr"))
}

/// MSIX packaging is Windows-only.
pub fn is_packaged() -> bool {
    false
}

pub fn is_root() -> bool {
    // SAFETY: geteuid has no preconditions and cannot fail.
    unsafe { libc::geteuid() == 0 }
}
