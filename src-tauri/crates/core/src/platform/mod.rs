//! OS-specific helpers behind one interface:
//! `installer_language()`, `idle_ms()`, `ui_language_is_turkish()`, `is_packaged()`.
//! `unsafe` is confined to this module, `net/` and `proc/`.

#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "linux")]
pub use linux::*;

#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "macos")]
pub use macos::*;

#[cfg(windows)]
mod windows;
#[cfg(windows)]
pub use windows::*;

/// Human-readable OS name for feedback reports, e.g. "Windows 11 Home 10.0.26200".
pub fn os_description() -> String {
    sysinfo::System::long_os_version().unwrap_or_else(|| std::env::consts::OS.to_string())
}

/// Effective user is root (Linux, macOS): only then are other users' sockets and processes visible.
#[cfg(unix)]
pub fn is_root() -> bool {
    // SAFETY: geteuid has no preconditions and cannot fail.
    unsafe { libc::geteuid() == 0 }
}
