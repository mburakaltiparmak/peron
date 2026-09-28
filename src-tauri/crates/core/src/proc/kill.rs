//! Process termination. Callers must only reach this after explicit user confirmation.

use serde::Serialize;
use sysinfo::{Pid, System};

use crate::classify;
use crate::error::{AppError, AppResult};
use crate::proc::info;

#[cfg(windows)]
fn terminate(pid: u32) -> AppResult<()> {
    use windows::Win32::Foundation::{CloseHandle, HANDLE};
    use windows::Win32::System::Threading::{OpenProcess, TerminateProcess, PROCESS_TERMINATE};

    struct HandleGuard(HANDLE);
    impl Drop for HandleGuard {
        fn drop(&mut self) {
            // SAFETY: the handle was returned by OpenProcess and is closed exactly once.
            unsafe {
                let _ = CloseHandle(self.0);
            }
        }
    }

    // SAFETY: plain Win32 calls; the handle is owned by HandleGuard.
    unsafe {
        let handle = OpenProcess(PROCESS_TERMINATE, false, pid)?;
        let _guard = HandleGuard(handle);
        TerminateProcess(handle, 1)?;
    }
    Ok(())
}

/// SIGTERM first so dev servers can shut down cleanly; SIGKILL if still alive after 1.5 s.
#[cfg(target_os = "linux")]
fn terminate(pid: u32) -> AppResult<()> {
    fn signal(pid: u32, sig: libc::c_int) -> AppResult<()> {
        // SAFETY: kill(2) has no memory-safety preconditions.
        if unsafe { libc::kill(pid as libc::pid_t, sig) } == 0 {
            return Ok(());
        }
        Err(match std::io::Error::last_os_error().raw_os_error() {
            Some(libc::EPERM) => AppError::AccessDenied,
            Some(libc::ESRCH) => AppError::NotFound,
            _ => AppError::Other(std::io::Error::last_os_error().to_string()),
        })
    }
    let alive = || std::path::Path::new(&format!("/proc/{pid}")).exists() && !is_zombie(pid);

    signal(pid, libc::SIGTERM)?;
    for _ in 0..15 {
        std::thread::sleep(std::time::Duration::from_millis(100));
        if !alive() {
            return Ok(());
        }
    }
    match signal(pid, libc::SIGKILL) {
        Err(AppError::NotFound) => Ok(()),
        other => other,
    }
}

/// An exited child that its parent hasn't reaped yet still has a /proc entry (state `Z`).
#[cfg(target_os = "linux")]
fn is_zombie(pid: u32) -> bool {
    std::fs::read_to_string(format!("/proc/{pid}/stat"))
        .ok()
        .and_then(|s| {
            s.rsplit_once(')')
                .map(|(_, rest)| rest.trim_start().starts_with('Z'))
        })
        .unwrap_or(false)
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct KillReport {
    pub killed: Vec<u32>,
    pub failed: Vec<KillFailure>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct KillFailure {
    pub pid: u32,
    pub name: String,
    pub message: String,
}

/// Terminates `pid` (and optionally its descendants).
/// `expected_start_ms` is the process start time the UI showed, so a reused PID is never killed.
pub fn kill(
    sys: &mut System,
    pid: u32,
    expected_start_ms: i64,
    tree: bool,
) -> AppResult<KillReport> {
    info::refresh_all_light(sys);
    let process = sys.process(Pid::from_u32(pid)).ok_or(AppError::NotFound)?;
    let name = process.name().to_string_lossy().into_owned();
    if (process.start_time() as i64 * 1000 - expected_start_ms).abs() > 2000 {
        return Err(AppError::Changed);
    }
    if classify::is_protected(pid, &name) {
        return Err(AppError::Protected(name));
    }

    let mut report = KillReport {
        killed: Vec::new(),
        failed: Vec::new(),
    };
    if tree {
        for (child, child_name) in info::descendants(sys, pid) {
            if classify::is_protected(child, &child_name) {
                continue;
            }
            match terminate(child) {
                Ok(()) => report.killed.push(child),
                Err(AppError::NotFound) => {}
                Err(e) => report.failed.push(KillFailure {
                    pid: child,
                    name: child_name,
                    message: e.to_string(),
                }),
            }
        }
    }
    terminate(pid)?;
    report.killed.push(pid);
    Ok(report)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::net::ports::scan_all;

    fn start_ms(sys: &System, pid: u32) -> i64 {
        sys.process(Pid::from_u32(pid)).unwrap().start_time() as i64 * 1000
    }

    #[test]
    fn refuses_protected_and_stale_pid() {
        let mut sys = System::new();
        info::refresh_all_light(&mut sys);
        // Windows "System" is PID 4; on Linux PID 1 (init / container entrypoint) always exists.
        let protected_pid = if cfg!(windows) { 4 } else { 1 };
        let system_start = start_ms(&sys, protected_pid);
        assert!(matches!(
            kill(&mut sys, protected_pid, system_start, false),
            Err(AppError::Protected(_))
        ));
        let me = std::process::id();
        assert!(matches!(
            kill(&mut sys, me, 0, false),
            Err(AppError::Changed)
        ));
    }

    /// Spawns a real listener and closes it. Needs Python on PATH: `cargo test -- --ignored`.
    #[test]
    #[ignore]
    fn kills_listener_and_frees_port() {
        const PORT: u16 = 18799;
        let python = if cfg!(windows) { "python" } else { "python3" };
        let mut child = std::process::Command::new(python)
            .args(["-m", "http.server", &PORT.to_string()])
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn()
            .expect("python");
        let pid = child.id();
        let listening = || {
            scan_all()
                .unwrap()
                .iter()
                .any(|s| s.local_port == PORT && s.pid == pid)
        };
        // A cold Python start on a CI runner can take several seconds.
        for _ in 0..300 {
            if listening() {
                break;
            }
            if let Ok(Some(status)) = child.try_wait() {
                panic!("python exited before listening on {PORT}: {status}");
            }
            std::thread::sleep(std::time::Duration::from_millis(100));
        }
        assert!(listening(), "listener did not start within 30 s");

        let mut sys = System::new();
        info::refresh_all_light(&mut sys);
        let started = start_ms(&sys, pid);
        let report = kill(&mut sys, pid, started, true).expect("kill");
        assert!(report.killed.contains(&pid));
        let _ = child.wait(); // reap the terminated child
        std::thread::sleep(std::time::Duration::from_millis(300));
        assert!(!listening(), "port still open");
    }
}
