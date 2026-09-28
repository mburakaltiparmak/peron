//! Client-side send limit for the feedback form (part of the bot protection: 3 per hour,
//! 10 per day). Timestamps persist in the app config dir so restarting the app doesn't reset them.
//! Server-side filtering is done by the mail relay (Web3Forms).

use std::path::PathBuf;

use tauri::{AppHandle, Manager};

use crate::error::{AppError, AppResult};
use crate::state::now_ms;
use crate::util;

const HOUR_MS: i64 = 60 * 60 * 1000;
const DAY_MS: i64 = 24 * HOUR_MS;
const PER_HOUR: usize = 3;
const PER_DAY: usize = 10;

fn path(app: &AppHandle) -> Option<PathBuf> {
    app.path()
        .app_config_dir()
        .ok()
        .map(|d| d.join("feedback.json"))
}

/// Returns minutes to wait, or `None` if a send is allowed now. `sent` is pruned to the last day.
fn check(sent: &mut Vec<i64>, now: i64) -> Option<u64> {
    sent.retain(|t| now - t < DAY_MS);
    let last_hour: Vec<i64> = sent.iter().copied().filter(|t| now - t < HOUR_MS).collect();
    let wait_until = if sent.len() >= PER_DAY {
        sent.iter().min().map(|t| t + DAY_MS)
    } else if last_hour.len() >= PER_HOUR {
        last_hour.iter().min().map(|t| t + HOUR_MS)
    } else {
        None
    }?;
    Some(((wait_until - now).max(0) as u64).div_ceil(60_000).max(1))
}

/// Reserves one send slot (counts attempts, so a failing relay can't be hammered either).
pub fn reserve(app: &AppHandle) -> AppResult<()> {
    let path = path(app);
    let mut sent: Vec<i64> = path
        .as_ref()
        .and_then(|p| util::read_with_backup(p, |s| serde_json::from_str(s).ok()))
        .unwrap_or_default();
    let now = now_ms();
    if let Some(minutes) = check(&mut sent, now) {
        return Err(AppError::RateLimited(minutes));
    }
    sent.push(now);
    if let Some(p) = path {
        if let Err(e) = util::write_atomic(
            &p,
            serde_json::to_string(&sent).unwrap_or_default().as_bytes(),
        ) {
            crate::log_error!("feedback limit not saved: {e}");
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn limits() {
        let now = 10 * DAY_MS;
        let mut sent = vec![now - 10_000, now - 20_000];
        assert_eq!(check(&mut sent, now), None);
        sent.push(now - 30_000);
        // 3 in the last hour → wait until the oldest of them is an hour old (~60 min).
        assert_eq!(check(&mut sent, now), Some(60));

        let mut day: Vec<i64> = (0..10).map(|i| now - (i + 2) * HOUR_MS).collect();
        assert!(check(&mut day, now).is_some());

        let mut old = vec![now - 2 * DAY_MS; 20];
        assert_eq!(check(&mut old, now), None);
        assert!(old.is_empty());
    }
}
