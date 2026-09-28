//! Local port event history: `<app local data>/history.jsonl`, one JSON event per line.
//! Memory (what the UI and exports see) holds at most 30 days / 2,000 events; the file is only
//! appended to and gets rewritten from memory once it reaches 2,500 lines.
//! Never leaves the device (see PRIVACY.md).

use std::io::Write;
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};

use peron_core::history::PortEvent;
use tauri::{AppHandle, Manager};

use crate::state::now_ms;
use crate::util;

pub const MAX_EVENTS: usize = 2_000;
const COMPACT_AT: usize = 2_500;
const MAX_AGE_MS: i64 = 30 * 24 * 60 * 60 * 1000;
/// Lines currently in history.jsonl (memory is trimmed more eagerly than the file).
static FILE_LINES: AtomicUsize = AtomicUsize::new(0);

fn path(app: &AppHandle) -> Option<PathBuf> {
    app.path()
        .app_local_data_dir()
        .ok()
        .map(|d| d.join("history.jsonl"))
}

fn trim(events: &mut Vec<PortEvent>, now: i64) {
    events.retain(|e| now - e.at_ms < MAX_AGE_MS);
    if events.len() > MAX_EVENTS {
        events.drain(..events.len() - MAX_EVENTS);
    }
}

/// Oldest first. Corrupt lines (e.g. a crash mid-append) are skipped.
pub fn load(app: &AppHandle) -> Vec<PortEvent> {
    let Some(text) = path(app).and_then(|p| std::fs::read_to_string(p).ok()) else {
        FILE_LINES.store(0, Ordering::Relaxed);
        return Vec::new();
    };
    FILE_LINES.store(text.lines().count(), Ordering::Relaxed);
    let mut events: Vec<PortEvent> = text
        .lines()
        .filter_map(|l| serde_json::from_str(l).ok())
        .collect();
    trim(&mut events, now_ms());
    events
}

/// Appends new events to memory and disk. `all` is the in-memory list (oldest first).
pub fn append(app: &AppHandle, all: &mut Vec<PortEvent>, new: &[PortEvent]) {
    all.extend_from_slice(new);
    trim(all, now_ms());
    let Some(p) = path(app) else { return };
    let lines = FILE_LINES.load(Ordering::Relaxed) + new.len();
    let result = if lines >= COMPACT_AT {
        FILE_LINES.store(all.len(), Ordering::Relaxed);
        let body: String = all
            .iter()
            .filter_map(|e| serde_json::to_string(e).ok())
            .map(|l| l + "\n")
            .collect();
        util::write_atomic(&p, body.as_bytes())
    } else {
        FILE_LINES.store(lines, Ordering::Relaxed);
        (|| {
            if let Some(dir) = p.parent() {
                std::fs::create_dir_all(dir)?;
            }
            let mut f = std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(&p)?;
            for e in new {
                writeln!(f, "{}", serde_json::to_string(e).unwrap_or_default())?;
            }
            Ok(())
        })()
    };
    if let Err(e) = result {
        crate::log_error!("history not saved: {e}");
    }
}

pub fn clear(app: &AppHandle) {
    FILE_LINES.store(0, Ordering::Relaxed);
    if let Some(p) = path(app) {
        let _ = std::fs::remove_file(p);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use peron_core::history::EventKind;

    fn ev(at_ms: i64) -> PortEvent {
        PortEvent {
            at_ms,
            kind: EventKind::Opened,
            port: 3000,
            pid: 1,
            process: "node".into(),
            project: None,
            addrs: vec!["127.0.0.1".into()],
            exposed: false,
            is_system: false,
            is_dev: true,
        }
    }

    #[test]
    fn trim_caps_count_and_age() {
        let now = MAX_AGE_MS * 2;
        let mut events: Vec<_> = (0..MAX_EVENTS as i64 + 10)
            .map(|i| ev(now - i))
            .rev()
            .collect();
        events.insert(0, ev(now - MAX_AGE_MS - 1));
        trim(&mut events, now);
        assert_eq!(events.len(), MAX_EVENTS);
        // The newest events are kept.
        assert_eq!(events.last().unwrap().at_ms, now);
        assert!(events.iter().all(|e| now - e.at_ms < MAX_AGE_MS));
    }
}
