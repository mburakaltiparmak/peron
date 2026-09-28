//! peron-core: everything Peron knows about ports and processes, without any GUI.
//! Used by the desktop app (Tauri) and by `peron-cli` (headless servers, SSH remote view).

pub mod classify;
pub mod error;
pub mod export;
pub mod history;
pub mod i18n;
pub mod model;
pub mod net;
pub mod platform;
pub mod proc;
pub mod remote;
pub mod util;
