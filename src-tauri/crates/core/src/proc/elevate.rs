//! Elevation helpers. The Windows manifest stays `asInvoker`; elevation is always user-initiated.
//! On Linux there is no in-app elevation (GUI apps under pkexec/sudo are unreliable on Wayland);
//! the UI tells the user to start Peron as root instead.

use crate::error::{AppError, AppResult};

/// Passed to the elevated instance so it waits for this one to exit (single-instance lock).
pub const RELAUNCH_ARG: &str = "--relaunched";

/// Whether `relaunch_as_admin` is offered: Windows, unpackaged only. MSIX (Store) apps would need
/// the restricted `allowElevation` capability, so the Store build simply lists what the user can
/// see and asks nothing more.
pub fn can_elevate() -> bool {
    cfg!(windows) && !crate::platform::is_packaged()
}

#[cfg(windows)]
pub fn is_elevated() -> bool {
    // SAFETY: no arguments, no side effects.
    unsafe { windows::Win32::UI::Shell::IsUserAnAdmin().as_bool() }
}

#[cfg(target_os = "linux")]
pub fn is_elevated() -> bool {
    crate::platform::is_root()
}

/// Starts an elevated copy of this executable via the UAC prompt.
#[cfg(windows)]
pub fn relaunch_as_admin() -> AppResult<()> {
    use windows::core::{w, HSTRING, PCWSTR};
    use windows::Win32::UI::Shell::ShellExecuteW;
    use windows::Win32::UI::WindowsAndMessaging::SW_SHOWNORMAL;

    let exe = std::env::current_exe().map_err(|e| AppError::Other(e.to_string()))?;
    let exe = HSTRING::from(exe.as_os_str());
    let args = HSTRING::from(RELAUNCH_ARG);
    // SAFETY: all strings outlive the call.
    let result = unsafe {
        ShellExecuteW(
            None,
            w!("runas"),
            &exe,
            &args,
            PCWSTR::null(),
            SW_SHOWNORMAL,
        )
    };
    // Values <= 32 are errors; the user declining UAC also lands here.
    if result.0 as isize <= 32 {
        return Err(AppError::Other(
            "Yönetici olarak başlatılamadı (UAC onayı verilmedi).".into(),
        ));
    }
    Ok(())
}

#[cfg(target_os = "linux")]
pub fn relaunch_as_admin() -> AppResult<()> {
    Err(AppError::Other(
        "Not supported on Linux; start Peron as root.".into(),
    ))
}
