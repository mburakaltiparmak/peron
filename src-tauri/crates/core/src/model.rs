//! `PortEntry`: a socket joined with its owning process, as sent to the UI.

use std::collections::HashMap;
use std::net::IpAddr;

use serde::{Deserialize, Serialize};
use sysinfo::System;

use crate::classify;
use crate::error::AppResult;
use crate::net::ports::{self, Protocol};
use crate::proc::info::{self, Detail, ProcInfo};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PortEntry {
    pub id: String,
    pub protocol: Protocol,
    pub ip_version: u8,
    pub local_addr: String,
    pub local_port: u16,
    pub remote_addr: Option<String>,
    pub remote_port: Option<u16>,
    /// TCP state, or "BOUND" for UDP.
    pub state: String,
    pub pid: u32,
    pub process_name: String,
    pub exe_path: Option<String>,
    pub cmdline: Option<String>,
    pub cwd: Option<String>,
    /// Project name = last folder of `cwd` (see `project_of`); `None` when it says nothing.
    #[serde(default)]
    pub project: Option<String>,
    pub parent_pid: Option<u32>,
    pub parent_name: Option<String>,
    pub opened_at_ms: Option<i64>,
    /// "socket" when taken from the socket table, "process" when it fell back to process start.
    pub opened_at_source: String,
    pub process_start_ms: i64,
    pub cpu_percent: f32,
    pub memory_bytes: u64,
    pub disk_read_bytes: u64,
    pub disk_written_bytes: u64,
    pub is_system: bool,
    pub is_protected: bool,
    pub is_dev: bool,
    pub is_localhost_only: bool,
    pub accessible: bool,
}

impl PortEntry {
    pub fn is_listening(&self) -> bool {
        self.protocol == Protocol::Tcp && self.state == "LISTEN"
    }
}

fn fmt_addr(a: &IpAddr) -> String {
    match a {
        IpAddr::V4(v4) => v4.to_string(),
        IpAddr::V6(v6) => format!("[{v6}]"),
    }
}

pub fn build_snapshot(sys: &mut System, detail: Detail) -> AppResult<Vec<PortEntry>> {
    let sockets = ports::scan_all()?;
    let pids: Vec<u32> = sockets.iter().map(|s| s.pid).collect();
    info::refresh_pids(sys, &pids, detail);
    let cpus = info::cpu_count();
    let mut cache: HashMap<u32, Option<ProcInfo>> = HashMap::new();

    let mut entries: Vec<PortEntry> = sockets
        .into_iter()
        .map(|s| {
            let proc = cache
                .entry(s.pid)
                .or_insert_with(|| info::lookup(sys, s.pid, cpus))
                .clone();
            let name = proc
                .as_ref()
                .map(|p| p.name.clone())
                .unwrap_or_else(|| match s.pid {
                    // Windows: TIME_WAIT etc. Linux: owner not visible without root.
                    0 if cfg!(windows) => "System Idle Process".into(),
                    0 => "?".into(),
                    4 if cfg!(windows) => "System".into(),
                    _ => format!("PID {}", s.pid),
                });
            let exe = proc.as_ref().and_then(|p| p.exe.clone());
            let process_start_ms = proc.as_ref().map(|p| p.start_time_ms).unwrap_or(0);
            let (opened_at_ms, opened_at_source) = match s.created_at_ms {
                Some(t) => (Some(t), "socket".to_string()),
                None => (
                    (process_start_ms > 0).then_some(process_start_ms),
                    "process".to_string(),
                ),
            };
            let state = s.state.unwrap_or("BOUND").to_string();
            let id = format!(
                "{:?}|{}|{}|{}|{}|{}",
                s.protocol,
                s.local_addr,
                s.local_port,
                s.remote_addr.map(|a| a.to_string()).unwrap_or_default(),
                s.remote_port.unwrap_or(0),
                s.pid
            );
            PortEntry {
                id,
                protocol: s.protocol,
                ip_version: if s.local_addr.is_ipv4() { 4 } else { 6 },
                local_addr: fmt_addr(&s.local_addr),
                local_port: s.local_port,
                remote_addr: s.remote_addr.as_ref().map(fmt_addr),
                remote_port: s.remote_port,
                state,
                pid: s.pid,
                is_system: classify::is_system(s.pid, &name, exe.as_deref()),
                is_protected: classify::is_protected(s.pid, &name),
                is_dev: classify::is_dev(&name),
                is_localhost_only: s.local_addr.is_loopback(),
                process_name: name,
                exe_path: exe,
                cmdline: proc.as_ref().and_then(|p| p.cmdline.clone()),
                cwd: proc.as_ref().and_then(|p| p.cwd.clone()),
                project: proc
                    .as_ref()
                    .and_then(|p| project_of(p.cwd.as_deref(), p.exe.as_deref())),
                parent_pid: proc.as_ref().and_then(|p| p.parent_pid),
                parent_name: proc.as_ref().and_then(|p| p.parent_name.clone()),
                opened_at_ms,
                opened_at_source,
                process_start_ms,
                cpu_percent: proc.as_ref().map(|p| p.cpu_percent).unwrap_or(0.0),
                memory_bytes: proc.as_ref().map(|p| p.memory_bytes).unwrap_or(0),
                disk_read_bytes: proc.as_ref().map(|p| p.disk_read_bytes).unwrap_or(0),
                disk_written_bytes: proc.as_ref().map(|p| p.disk_written_bytes).unwrap_or(0),
                accessible: proc.as_ref().is_some_and(|p| p.accessible),
            }
        })
        .collect();

    entries.sort_by(|a, b| {
        a.local_port
            .cmp(&b.local_port)
            .then(a.protocol.cmp_key().cmp(&b.protocol.cmp_key()))
    });
    dedupe_ids(&mut entries);
    Ok(entries)
}

/// Project name = last folder of the working directory, e.g. `C:\code\shop-api` → `shop-api`.
/// `None` for folders that say nothing about a project: drive/file-system roots, home folders,
/// OS and program directories, or the app's own install folder (VS Code running from its dir).
pub fn project_of(cwd: Option<&str>, exe: Option<&str>) -> Option<String> {
    let path = cwd?.trim_end_matches(['\\', '/']);
    if path.is_empty() {
        return None;
    }
    let lower = path.to_ascii_lowercase().replace('/', "\\");
    if let Some(exe) = exe {
        let exe = exe.to_ascii_lowercase().replace('/', "\\");
        if exe.starts_with(&format!("{lower}\\")) {
            return None;
        }
    }
    let parts: Vec<&str> = lower.split('\\').filter(|p| !p.is_empty()).collect();
    let is_drive = parts.len() == 1 && parts[0].len() == 2 && parts[0].ends_with(':');
    let noise = is_drive
        || parts.is_empty()
        || matches!(
            parts.as_slice(),
            [_, "windows", ..] | [_, "program files", ..] | [_, "program files (x86)", ..]
        )
        || matches!(parts.as_slice(), [_, "users", _] | ["home", _] | ["root"])
        || matches!(
            parts.as_slice(),
            ["usr", ..] | ["bin"] | ["sbin"] | ["etc", ..] | ["opt"] | ["var"] | ["tmp"]
        );
    if noise {
        return None;
    }
    path.rsplit(['\\', '/'])
        .next()
        .filter(|s| !s.is_empty())
        .map(str::to_owned)
}

/// A process may bind the same address+port several times (SO_REUSEADDR, e.g. mDNS on UDP 5353).
fn dedupe_ids(entries: &mut [PortEntry]) {
    let mut seen: HashMap<String, u32> = HashMap::new();
    for e in entries.iter_mut() {
        let n = seen.entry(e.id.clone()).or_insert(0);
        if *n > 0 {
            e.id = format!("{}#{}", e.id, n);
        }
        *n += 1;
    }
}

impl Protocol {
    fn cmp_key(&self) -> u8 {
        match self {
            Protocol::Tcp => 0,
            Protocol::Udp => 1,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::project_of;

    #[test]
    fn projects() {
        assert_eq!(
            project_of(Some(r"C:\code\shop-api\"), None).as_deref(),
            Some("shop-api")
        );
        assert_eq!(
            project_of(Some("/home/u/work/blog"), None).as_deref(),
            Some("blog")
        );
        assert_eq!(project_of(Some("/srv/app"), None).as_deref(), Some("app"));
    }

    #[test]
    fn noise_folders() {
        for p in [
            r"C:\",
            r"C:\Windows\System32",
            r"C:\Users\burak",
            r"C:\Program Files\App",
            "/",
            "/home/u",
            "/root",
            "/usr/bin",
            "/tmp",
        ] {
            assert_eq!(project_of(Some(p), None), None, "{p}");
        }
        assert_eq!(project_of(None, None), None);
    }

    #[test]
    fn own_install_folder() {
        let vsc = r"C:\Users\u\AppData\Local\Programs\Microsoft VS Code";
        assert_eq!(
            project_of(Some(vsc), Some(&format!(r"{vsc}\Code.exe"))),
            None
        );
    }
}
