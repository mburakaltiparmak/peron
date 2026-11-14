//! macOS helpers mirroring `platform/linux.rs`.

/// No installer asks for a language on macOS; the system language list decides until the user
/// changes it in Settings.
pub fn installer_language() -> Option<String> {
    None
}

/// Seconds since the last keyboard/mouse/trackpad input in this login session, as ms.
pub fn idle_ms() -> u64 {
    // CoreGraphics: kCGEventSourceStateCombinedSessionState = 0, kCGAnyInputEventType = !0.
    #[link(name = "CoreGraphics", kind = "framework")]
    extern "C" {
        fn CGEventSourceSecondsSinceLastEventType(state: i32, event_type: u32) -> f64;
    }
    // SAFETY: plain value arguments, no pointers; returns 0 or a positive duration.
    let secs = unsafe { CGEventSourceSecondsSinceLastEventType(0, !0) };
    if secs.is_finite() && secs > 0.0 {
        (secs * 1000.0) as u64
    } else {
        0
    }
}

/// True when the first preferred system language is Turkish. Apps started from Finder/Dock have
/// no `LANG`, so this reads the user's language list (System Settings > General > Language).
pub fn ui_language_is_turkish() -> bool {
    sys_locale::get_locale().is_some_and(|l| l.to_ascii_lowercase().starts_with("tr"))
}

/// Store packaging is Windows-only.
pub fn is_packaged() -> bool {
    false
}
