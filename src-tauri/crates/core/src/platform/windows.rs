//! Windows helpers: registry strings, user idle time, UI language, installer language.

/// Written by the NSIS installer hook (`installer-hooks.nsh`).
const INSTALLER_KEY: &str = r"Software\Peron";
const INSTALLER_VALUE: &str = "InstallerLanguage";

/// Language picked in the installer ("tr"/"en"), if any.
pub fn installer_language() -> Option<String> {
    hkcu_get_string(INSTALLER_KEY, INSTALLER_VALUE)
}

use std::ffi::c_void;

use windows::core::{HSTRING, PCWSTR};
use windows::Win32::Foundation::ERROR_SUCCESS;
use windows::Win32::Globalization::GetUserDefaultUILanguage;
use windows::Win32::System::Registry::{
    RegCloseKey, RegCreateKeyExW, RegGetValueW, RegSetValueExW, HKEY, HKEY_CURRENT_USER, KEY_WRITE,
    REG_OPTION_NON_VOLATILE, REG_SZ, RRF_RT_REG_SZ,
};
use windows::Win32::System::SystemInformation::GetTickCount;
use windows::Win32::UI::Input::KeyboardAndMouse::{GetLastInputInfo, LASTINPUTINFO};

/// Writes string values under `HKCU\<subkey>`, creating the key if needed.
pub fn hkcu_set_strings(subkey: &str, values: &[(&str, &str)]) -> windows::core::Result<()> {
    let subkey = HSTRING::from(subkey);
    let mut key = HKEY::default();
    // SAFETY: out-pointer is a valid HKEY; the key is closed below.
    unsafe {
        RegCreateKeyExW(
            HKEY_CURRENT_USER,
            &subkey,
            None,
            PCWSTR::null(),
            REG_OPTION_NON_VOLATILE,
            KEY_WRITE,
            None,
            &mut key,
            None,
        )
        .ok()?;
    }
    let result = values.iter().try_for_each(|(name, value)| {
        let wide: Vec<u16> = value.encode_utf16().chain(std::iter::once(0)).collect();
        // SAFETY: `wide` is a NUL-terminated UTF-16 buffer that outlives the call.
        let bytes =
            unsafe { std::slice::from_raw_parts(wide.as_ptr() as *const u8, wide.len() * 2) };
        // SAFETY: `key` is open with KEY_WRITE access.
        unsafe { RegSetValueExW(key, &HSTRING::from(*name), None, REG_SZ, Some(bytes)).ok() }
    });
    // SAFETY: `key` was opened above and is closed exactly once.
    unsafe {
        let _ = RegCloseKey(key);
    }
    result
}

/// Reads a string value from `HKCU\<subkey>`.
pub fn hkcu_get_string(subkey: &str, name: &str) -> Option<String> {
    let (subkey, name) = (HSTRING::from(subkey), HSTRING::from(name));
    let mut buf = [0u16; 256];
    let mut size = (buf.len() * 2) as u32;
    // SAFETY: buffer pointer and size describe `buf`, which outlives the call.
    let status = unsafe {
        RegGetValueW(
            HKEY_CURRENT_USER,
            &subkey,
            &name,
            RRF_RT_REG_SZ,
            None,
            Some(buf.as_mut_ptr() as *mut c_void),
            Some(&mut size),
        )
    };
    if status != ERROR_SUCCESS {
        return None;
    }
    let len = (size as usize / 2).saturating_sub(1); // drop the terminating NUL
    Some(String::from_utf16_lossy(&buf[..len.min(buf.len())]))
}

/// Milliseconds since the last keyboard/mouse input in this session.
pub fn idle_ms() -> u64 {
    let mut info = LASTINPUTINFO {
        cbSize: std::mem::size_of::<LASTINPUTINFO>() as u32,
        dwTime: 0,
    };
    // SAFETY: `info` is a valid, size-initialized out-parameter.
    if !unsafe { GetLastInputInfo(&mut info) }.as_bool() {
        return 0;
    }
    // SAFETY: no arguments. Tick counts wrap every ~49.7 days; wrapping_sub handles it.
    unsafe { GetTickCount() }.wrapping_sub(info.dwTime) as u64
}

/// True when the Windows display language is Turkish (primary language id 0x1F).
pub fn ui_language_is_turkish() -> bool {
    // SAFETY: no arguments.
    let langid = unsafe { GetUserDefaultUILanguage() };
    langid & 0x3FF == 0x1F
}

// ---------------------------------------------------------------------------------------------
// MSIX package identity (Microsoft Store build). The same peron.exe runs unpackaged (NSIS,
// site/GitHub) or inside an MSIX package; behavior differences key off `is_packaged()`.

/// `STARTUP_TASK_ID` must match `<desktop:StartupTask TaskId=…>` in packaging/msix/AppxManifest.
const STARTUP_TASK_ID: &str = "PeronStartup";

/// True when running from an MSIX package (i.e. the Microsoft Store build).
pub fn is_packaged() -> bool {
    use windows::Win32::Storage::Packaging::Appx::GetCurrentPackageFullName;
    let mut len = 0u32;
    // SAFETY: querying the length only; a null buffer is allowed.
    let status = unsafe { GetCurrentPackageFullName(&mut len, None) };
    // APPMODEL_ERROR_NO_PACKAGE (15700) when unpackaged; ERROR_INSUFFICIENT_BUFFER when packaged.
    status.0 != 15700
}

/// The package's AppUserModelID (`<PackageFamilyName>!Peron`) for toast notifications.
pub fn package_aumid() -> Option<String> {
    use windows::core::PWSTR;
    use windows::Win32::Storage::Packaging::Appx::GetCurrentApplicationUserModelId;
    let mut len = 0u32;
    // SAFETY: first call gets the length; second fills a buffer of exactly that size.
    unsafe {
        let _ = GetCurrentApplicationUserModelId(&mut len, None);
        if len == 0 {
            return None;
        }
        let mut buf = vec![0u16; len as usize];
        if GetCurrentApplicationUserModelId(&mut len, Some(PWSTR(buf.as_mut_ptr())))
            != ERROR_SUCCESS
        {
            return None;
        }
        Some(String::from_utf16_lossy(
            &buf[..(len as usize).saturating_sub(1)],
        ))
    }
}

/// True when Windows started us through the MSIX StartupTask (sign-in autostart).
pub fn launched_by_startup_task() -> bool {
    use windows::ApplicationModel::Activation::ActivationKind;
    use windows::ApplicationModel::AppInstance;
    AppInstance::GetActivatedEventArgs()
        .and_then(|args| args.Kind())
        .is_ok_and(|kind| kind == ActivationKind::StartupTask)
}

fn startup_task() -> windows::core::Result<windows::ApplicationModel::StartupTask> {
    windows::ApplicationModel::StartupTask::GetAsync(&HSTRING::from(STARTUP_TASK_ID))?.join()
}

pub fn startup_task_enabled() -> bool {
    use windows::ApplicationModel::StartupTaskState;
    startup_task()
        .and_then(|t| t.State())
        .is_ok_and(|s| s == StartupTaskState::Enabled || s == StartupTaskState::EnabledByPolicy)
}

/// Enables/disables the MSIX StartupTask. Fails with a readable message when the user or policy
/// turned it off in Windows Settings (apps may not override that).
pub fn set_startup_task(enable: bool) -> Result<(), String> {
    use windows::ApplicationModel::StartupTaskState;
    let task = startup_task().map_err(|e| e.message().to_string())?;
    if !enable {
        task.Disable().map_err(|e| e.message().to_string())?;
        return Ok(());
    }
    let state = task
        .RequestEnableAsync()
        .and_then(|op| op.join())
        .map_err(|e| e.message().to_string())?;
    match state {
        StartupTaskState::Enabled | StartupTaskState::EnabledByPolicy => Ok(()),
        StartupTaskState::DisabledByUser => {
            Err("disabled in Windows Settings > Apps > Startup; enable it there".into())
        }
        _ => Err("disabled by policy".into()),
    }
}
