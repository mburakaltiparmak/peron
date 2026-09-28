//! Background scan loop. The scan interval and depth adapt to what the user can see, so that
//! Peron idles cheaply in the tray:
//!
//! | Mode       | When                         | Interval              | Detail |
//! |------------|------------------------------|-----------------------|--------|
//! | Active     | window visible + focused     | user setting (3 s)    | Full   |
//! | Passive    | window visible, unfocused    | max(setting, 10 s)    | Full   |
//! | Background | no window / minimized        | 2 min                 | Light  |
//! | Idle       | no input for 10 min          | 5 min                 | Light  |
//!
//! There is deliberately no separate power-saving mode: this policy is always the frugal one.

use std::sync::mpsc;
use std::time::Duration;

use sysinfo::System;
use tauri::{AppHandle, Emitter, Manager};

use crate::error::AppResult;
use crate::history::{self, EventKind};
use crate::i18n::{self, Text};
use crate::model::{self, PortEntry};
use crate::proc::info::Detail;
use crate::settings::Settings;
use crate::state::{now_ms, AppState, Mode, ModeInfo};
use crate::util::LockExt;
use crate::{history_store, notify, platform, tray};

pub const PORTS_UPDATED: &str = "ports-updated";
pub const MODE_CHANGED: &str = "mode-changed";
pub const HISTORY_UPDATED: &str = "history-updated";

const PASSIVE_MIN_SECS: u64 = 10;
const BACKGROUND_INTERVAL_SECS: u64 = 2 * 60;
const IDLE_AFTER_MS: u64 = 10 * 60 * 1000;
const IDLE_INTERVAL_SECS: u64 = 5 * 60;

pub fn spawn(app: AppHandle) {
    let (tx, rx) = mpsc::channel();
    *app.state::<AppState>().wake.lock_safe() = Some(tx);
    std::thread::spawn(move || loop {
        let info = current_mode(&app);
        let changed = {
            let state = app.state::<AppState>();
            let mut mode = state.mode.lock_safe();
            let changed = mode.mode != info.mode || mode.interval_secs != info.interval_secs;
            if changed && info.mode == Mode::Background && mode.mode != Mode::Idle {
                // Leaving the UI: drop cached process data to return memory.
                *state.sys.lock_safe() = System::new();
            }
            *mode = info;
            changed
        };
        if changed {
            let _ = app.emit(MODE_CHANGED, info);
        }

        let detail = match info.mode {
            Mode::Active | Mode::Passive => Detail::Full,
            Mode::Background | Mode::Idle => Detail::Light,
        };
        if let Err(e) = rescan(&app, detail) {
            crate::log_error!("scan failed: {e}");
        }

        // Sleep until the interval passes or someone wakes us; drain extra wake-ups.
        let _ = rx.recv_timeout(Duration::from_secs(info.interval_secs));
        while rx.try_recv().is_ok() {}
    });
}

fn current_mode(app: &AppHandle) -> ModeInfo {
    let settings = app.state::<AppState>().settings.lock_safe().clone();
    let window = app.get_webview_window("main");
    let visible = window
        .as_ref()
        .is_some_and(|w| w.is_visible().unwrap_or(false) && !w.is_minimized().unwrap_or(false));
    let focused = visible
        && window
            .as_ref()
            .is_some_and(|w| w.is_focused().unwrap_or(false));
    mode_for(&settings, visible, focused, platform::idle_ms())
}

/// Pure mode policy, unit-tested below.
fn mode_for(s: &Settings, visible: bool, focused: bool, idle_ms: u64) -> ModeInfo {
    let user = s.refresh_interval_secs as u64;
    let (mode, interval_secs) = if visible && focused {
        (Mode::Active, user)
    } else if idle_ms >= IDLE_AFTER_MS {
        (Mode::Idle, IDLE_INTERVAL_SECS)
    } else if visible {
        (Mode::Passive, user.max(PASSIVE_MIN_SECS))
    } else {
        (Mode::Background, BACKGROUND_INTERVAL_SECS)
    };
    // With auto refresh off, a visible window only updates on demand; keep reminders going.
    let interval_secs = if !s.auto_refresh && visible {
        interval_secs.max(BACKGROUND_INTERVAL_SECS)
    } else {
        interval_secs
    };
    ModeInfo {
        mode,
        interval_secs,
    }
}

/// Scans now, stores the snapshot and notifies every consumer.
pub fn rescan(app: &AppHandle, detail: Detail) -> AppResult<Vec<PortEntry>> {
    let state = app.state::<AppState>();
    let entries = {
        let mut sys = state.sys.lock_safe();
        model::build_snapshot(&mut sys, detail)?
    };
    *state.snapshot.lock_safe() = entries.clone();
    // Only a live window listens; skip serializing the list otherwise.
    if detail == Detail::Full && app.get_webview_window("main").is_some() {
        let _ = app.emit(PORTS_UPDATED, &entries);
    }
    tray::update(app, &entries);
    record_history(app, &state, &entries);
    remind(app, &state, &entries);
    Ok(entries)
}

/// Diffs against the previous scan, stores open/close events and alerts about listeners that
/// became reachable from the network. The first scan after start only sets the baseline.
fn record_history(app: &AppHandle, state: &AppState, entries: &[PortEntry]) {
    let events = {
        let mut baseline = state.baseline.lock_safe();
        let events = baseline
            .as_deref()
            .map(|prev| history::diff(prev, entries, now_ms()))
            .unwrap_or_default();
        *baseline = Some(entries.to_vec());
        events
    };
    if events.is_empty() {
        return;
    }
    history_store::append(app, &mut state.history.lock_safe(), &events);
    if app.get_webview_window("main").is_some() {
        let _ = app.emit(HISTORY_UPDATED, &events);
    }

    let settings = state.settings.lock_safe().clone();
    if !settings.alert_exposed {
        return;
    }
    let lang = settings.lang();
    for e in events
        .iter()
        .filter(|e| e.kind == EventKind::Opened && e.exposed && !e.is_system)
        .take(3)
    {
        let body = Text::ExposedBody {
            process: e.process.clone(),
            port: e.port,
            addrs: e.addrs.join(", "),
        };
        notify::show(
            app,
            &i18n::t(lang, Text::ExposedTitle),
            &i18n::t(lang, body),
        );
    }
}

fn remind(app: &AppHandle, state: &AppState, entries: &[PortEntry]) {
    let settings = state.settings.lock_safe().clone();
    let mut reminders = state.reminders.lock_safe();
    if !settings.reminder_enabled {
        reminders.clear();
        return;
    }
    let lang = settings.lang();
    let now = now_ms();
    let threshold = settings.reminder_threshold_minutes as i64 * 60_000;
    let repeat = settings.reminder_repeat_minutes as i64 * 60_000;

    let mut live_keys = Vec::new();
    let mut sent = 0;
    for e in entries
        .iter()
        .filter(|e| e.is_dev && !e.is_system && e.is_listening())
    {
        let Some(opened) = e.opened_at_ms else {
            continue;
        };
        // One reminder per process+port, regardless of IPv4/IPv6 duplicates.
        let key = format!("{}:{}:{}", e.pid, e.local_port, e.process_start_ms);
        live_keys.push(key.clone());
        let open_for = now - opened;
        if open_for < threshold || reminders.get(&key).is_some_and(|last| now - last < repeat) {
            continue;
        }
        reminders.insert(key, now);
        if sent >= 3 {
            continue; // avoid a notification storm; the rest are marked as reminded
        }
        sent += 1;
        let body = Text::ReminderBody {
            process: e.process_name.clone(),
            port: e.local_port,
            duration: i18n::duration(lang, open_for),
        };
        notify::show(
            app,
            &i18n::t(lang, Text::ReminderTitle),
            &i18n::t(lang, body),
        );
    }
    reminders.retain(|k, _| live_keys.contains(k));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn modes() {
        let s = Settings::default();
        let m = |v, f, idle| {
            let i = mode_for(&s, v, f, idle);
            (i.mode, i.interval_secs)
        };
        assert_eq!(m(true, true, 0), (Mode::Active, 3));
        assert_eq!(m(true, false, 0), (Mode::Passive, 10));
        assert_eq!(m(false, false, 0), (Mode::Background, 120));
        assert_eq!(m(false, false, IDLE_AFTER_MS), (Mode::Idle, 300));
        // Focused window wins over idle (e.g. user reading the list).
        assert_eq!(m(true, true, IDLE_AFTER_MS).0, Mode::Active);
    }

    #[test]
    fn manual_refresh_keeps_background_pace() {
        let s = Settings {
            auto_refresh: false,
            ..Settings::default()
        };
        assert_eq!(mode_for(&s, true, true, 0).interval_secs, 120);
    }
}
