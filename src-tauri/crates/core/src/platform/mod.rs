//! OS-specific helpers behind one interface:
//! `installer_language()`, `idle_ms()`, `ui_language_is_turkish()`.
//! `unsafe` is confined to this module, `net/` and `proc/`.

#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "linux")]
pub use linux::*;

#[cfg(windows)]
mod windows;
#[cfg(windows)]
pub use windows::*;

/// Human-readable OS name for feedback reports, e.g. "Windows 11 Home 10.0.26200".
pub fn os_description() -> String {
    sysinfo::System::long_os_version().unwrap_or_else(|| std::env::consts::OS.to_string())
}
