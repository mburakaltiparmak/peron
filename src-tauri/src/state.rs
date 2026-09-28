use std::collections::HashMap;
use std::sync::mpsc::Sender;
use std::sync::Mutex;

use serde::Serialize;
use sysinfo::System;

use crate::model::PortEntry;
use crate::settings::Settings;
use crate::util::LockExt;
use peron_core::export::Snapshot;
use peron_core::history::PortEvent;

/// How aggressively the monitor scans (see the table in monitor.rs).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Mode {
    /// Window visible and focused.
    Active,
    /// Window visible, not focused.
    Passive,
    /// No window, or minimized: tray + reminders only.
    Background,
    /// User away from the keyboard.
    Idle,
}

#[derive(Clone, Copy, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ModeInfo {
    pub mode: Mode,
    pub interval_secs: u64,
}

pub struct AppState {
    pub sys: Mutex<System>,
    pub snapshot: Mutex<Vec<PortEntry>>,
    pub settings: Mutex<Settings>,
    /// Reminder key → last notification time (Unix ms).
    pub reminders: Mutex<HashMap<String, i64>>,
    /// Last tray menu signature, to avoid rebuilding an unchanged menu.
    pub tray_signature: Mutex<String>,
    pub mode: Mutex<ModeInfo>,
    /// Wakes the monitor loop early (window shown, settings changed).
    pub wake: Mutex<Option<Sender<()>>>,
    /// Last snapshot fetched per remote host id (SSH view; used for export and kill checks).
    pub remote: Mutex<HashMap<String, Snapshot>>,
    /// Port open/close events, oldest first (see history_store).
    pub history: Mutex<Vec<PortEvent>>,
    /// Previous scan to diff against; `None` until the first scan (which only sets the baseline).
    pub baseline: Mutex<Option<Vec<PortEntry>>>,
}

impl AppState {
    pub fn new(settings: Settings) -> Self {
        Self {
            sys: Mutex::new(System::new()),
            snapshot: Mutex::new(Vec::new()),
            settings: Mutex::new(settings),
            reminders: Mutex::new(HashMap::new()),
            tray_signature: Mutex::new(String::new()),
            mode: Mutex::new(ModeInfo {
                mode: Mode::Background,
                interval_secs: 120,
            }),
            wake: Mutex::new(None),
            remote: Mutex::new(HashMap::new()),
            history: Mutex::new(Vec::new()),
            baseline: Mutex::new(None),
        }
    }

    pub fn wake_monitor(&self) {
        if let Some(tx) = self.wake.lock_safe().as_ref() {
            let _ = tx.send(());
        }
    }
}

pub use peron_core::util::now_ms;
