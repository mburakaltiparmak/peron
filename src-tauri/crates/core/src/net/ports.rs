//! Platform-neutral socket model; the OS-specific scanners live in `windows.rs` / `linux.rs`.

use std::net::IpAddr;

use serde::{Deserialize, Serialize};

use crate::error::AppResult;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum Protocol {
    Tcp,
    Udp,
}

#[derive(Clone, Debug)]
pub struct RawSocket {
    pub protocol: Protocol,
    pub local_addr: IpAddr,
    pub local_port: u16,
    pub remote_addr: Option<IpAddr>,
    pub remote_port: Option<u16>,
    /// TCP state name; `None` for UDP.
    pub state: Option<&'static str>,
    /// Owning process; 0 when unknown (Linux: socket owned by another user without root).
    pub pid: u32,
    /// Socket creation time, Unix ms. Windows only (`liCreateTimestamp`); Linux has no equivalent.
    pub created_at_ms: Option<i64>,
}

/// Every TCP (all states) and UDP socket for IPv4 and IPv6.
pub fn scan_all() -> AppResult<Vec<RawSocket>> {
    #[cfg(windows)]
    return super::windows::scan();
    #[cfg(target_os = "linux")]
    return super::linux::scan();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_own_listener() {
        let listener = std::net::TcpListener::bind("127.0.0.1:0").expect("bind");
        let port = listener.local_addr().unwrap().port();
        let sockets = scan_all().expect("scan");
        let own = sockets
            .iter()
            .find(|s| s.protocol == Protocol::Tcp && s.local_port == port)
            .expect("own listener in the table");
        assert_eq!(own.state, Some("LISTEN"));
        assert_eq!(own.pid, std::process::id());
    }
}
