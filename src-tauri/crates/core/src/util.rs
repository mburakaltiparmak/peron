//! Small shared helpers: crash-safe file writes and poison-tolerant locks.

use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, MutexGuard};

/// Current time as Unix milliseconds.
pub fn now_ms() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}

/// `2026-09-25T18:03:04Z` without a date/time dependency (civil-from-days, Howard Hinnant).
pub fn utc_timestamp(ms: i64) -> String {
    let secs = ms.div_euclid(1000);
    let (days, rem) = (secs.div_euclid(86_400), secs.rem_euclid(86_400));
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = yoe + era * 400 + i64::from(m <= 2);
    format!(
        "{y:04}-{m:02}-{d:02}T{:02}:{:02}:{:02}Z",
        rem / 3600,
        (rem % 3600) / 60,
        rem % 60
    )
}

/// Writes `bytes` so that `path` is either the old or the new content, never half-written:
/// temp file in the same folder → flush + fsync → rename over the target (atomic on NTFS/ext4).
/// The previous content is kept as `<name>.bak` for recovery.
pub fn write_atomic(path: &Path, bytes: &[u8]) -> std::io::Result<()> {
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir)?;
    }
    let tmp = sibling(path, "tmp");
    {
        let mut f = std::fs::File::create(&tmp)?;
        f.write_all(bytes)?;
        f.sync_all()?;
    }
    if path.exists() {
        let _ = std::fs::copy(path, sibling(path, "bak"));
    }
    std::fs::rename(&tmp, path)
}

/// Reads `path`, falling back to the `.bak` written by `write_atomic` when the main file is
/// missing or fails `parse` (e.g. corrupted by an older, non-atomic writer).
pub fn read_with_backup<T>(path: &Path, parse: impl Fn(&str) -> Option<T>) -> Option<T> {
    std::fs::read_to_string(path)
        .ok()
        .and_then(|s| parse(&s))
        .or_else(|| {
            std::fs::read_to_string(sibling(path, "bak"))
                .ok()
                .and_then(|s| parse(&s))
        })
}

fn sibling(path: &Path, ext: &str) -> PathBuf {
    let mut name = path.file_name().unwrap_or_default().to_os_string();
    name.push(".");
    name.push(ext);
    path.with_file_name(name)
}

/// `Mutex::lock` that survives poisoning. A panic while holding one of our locks leaves plain
/// data (snapshots, settings, caches) that is still usable, so recovering beats crashing
/// every later caller.
pub trait LockExt<T> {
    fn lock_safe(&self) -> MutexGuard<'_, T>;
}

impl<T> LockExt<T> for Mutex<T> {
    fn lock_safe(&self) -> MutexGuard<'_, T> {
        self.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn atomic_write_and_backup() {
        let dir = std::env::temp_dir().join(format!("peron-util-{}", std::process::id()));
        let file = dir.join("settings.json");
        write_atomic(&file, b"{\"a\":1}").unwrap();
        write_atomic(&file, b"{\"a\":2}").unwrap();
        assert_eq!(std::fs::read_to_string(&file).unwrap(), "{\"a\":2}");
        assert_eq!(
            std::fs::read_to_string(sibling(&file, "bak")).unwrap(),
            "{\"a\":1}"
        );

        // Corrupt main file → backup is used.
        std::fs::write(&file, b"{broken").unwrap();
        let parse = |s: &str| serde_json::from_str::<serde_json::Value>(s).ok();
        assert_eq!(read_with_backup(&file, parse).unwrap()["a"], 1);
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn timestamps() {
        assert_eq!(utc_timestamp(0), "1970-01-01T00:00:00Z");
        assert_eq!(utc_timestamp(1_704_067_200_000), "2024-01-01T00:00:00Z");
        assert_eq!(utc_timestamp(1_790_359_384_000), "2026-09-25T18:03:04Z");
    }

    #[test]
    fn poisoned_lock_recovers() {
        let m = std::sync::Arc::new(Mutex::new(5));
        let m2 = m.clone();
        let _ = std::thread::spawn(move || {
            let _g = m2.lock().unwrap();
            panic!("poison");
        })
        .join();
        assert!(m.is_poisoned());
        assert_eq!(*m.lock_safe(), 5);
    }
}
