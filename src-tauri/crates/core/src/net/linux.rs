//! Linux socket tables from `/proc/net/{tcp,tcp6,udp,udp6}`, owners via `/proc/<pid>/fd`.
//!
//! Owners of other users' sockets are only visible to root; those rows get pid 0 (unknown).
//! The kernel exposes no socket creation time, so `created_at_ms` is always `None` and the UI
//! falls back to the process start time.

use std::collections::HashMap;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

use super::ports::{Protocol, RawSocket};
use crate::error::AppResult;

pub fn scan() -> AppResult<Vec<RawSocket>> {
    let owners = socket_owners();
    let mut out = Vec::new();
    for (file, protocol, v6) in [
        ("/proc/net/tcp", Protocol::Tcp, false),
        ("/proc/net/tcp6", Protocol::Tcp, true),
        ("/proc/net/udp", Protocol::Udp, false),
        ("/proc/net/udp6", Protocol::Udp, true),
    ] {
        // A missing file just means the protocol family is disabled (e.g. no IPv6).
        let Ok(text) = std::fs::read_to_string(file) else {
            continue;
        };
        out.extend(
            text.lines()
                .skip(1)
                .filter_map(|l| parse_line(l, protocol, v6, &owners)),
        );
    }
    Ok(out)
}

/// socket inode → pid, from the `socket:[inode]` links in every readable `/proc/<pid>/fd`.
fn socket_owners() -> HashMap<u64, u32> {
    let mut owners = HashMap::new();
    let Ok(procs) = std::fs::read_dir("/proc") else {
        return owners;
    };
    for entry in procs.flatten() {
        let Some(pid) = entry
            .file_name()
            .to_str()
            .and_then(|s| s.parse::<u32>().ok())
        else {
            continue;
        };
        let Ok(fds) = std::fs::read_dir(entry.path().join("fd")) else {
            continue;
        };
        for fd in fds.flatten() {
            if let Some(inode) = std::fs::read_link(fd.path())
                .ok()
                .and_then(|t| t.to_str().and_then(parse_socket_link))
            {
                owners.entry(inode).or_insert(pid);
            }
        }
    }
    owners
}

fn parse_socket_link(target: &str) -> Option<u64> {
    target
        .strip_prefix("socket:[")?
        .strip_suffix(']')?
        .parse()
        .ok()
}

/// One row: `sl local_address rem_address st tx:rx tr:tm retrnsmt uid timeout inode ...`
fn parse_line(
    line: &str,
    protocol: Protocol,
    v6: bool,
    owners: &HashMap<u64, u32>,
) -> Option<RawSocket> {
    let cols: Vec<&str> = line.split_whitespace().collect();
    if cols.len() < 10 {
        return None;
    }
    let (local_addr, local_port) = parse_endpoint(cols[1], v6)?;
    let (remote_addr, remote_port) = parse_endpoint(cols[2], v6)?;
    let st = u8::from_str_radix(cols[3], 16).ok()?;
    let inode: u64 = cols[9].parse().ok()?;
    let (state, listening) = match protocol {
        Protocol::Tcp => {
            let s = tcp_state_name(st);
            (Some(s), s == "LISTEN")
        }
        Protocol::Udp => (None, true),
    };
    Some(RawSocket {
        protocol,
        local_addr,
        local_port,
        remote_addr: (!listening).then_some(remote_addr),
        remote_port: (!listening).then_some(remote_port),
        state,
        pid: owners.get(&inode).copied().unwrap_or(0),
        created_at_ms: None,
    })
}

/// `0100007F:0BB8` → 127.0.0.1:3000. Addresses are 32-bit words in host byte order.
fn parse_endpoint(s: &str, v6: bool) -> Option<(IpAddr, u16)> {
    let (addr, port) = s.split_once(':')?;
    let port = u16::from_str_radix(port, 16).ok()?;
    let addr = if v6 {
        if addr.len() != 32 {
            return None;
        }
        let mut bytes = [0u8; 16];
        for i in 0..4 {
            let word = u32::from_str_radix(&addr[i * 8..i * 8 + 8], 16).ok()?;
            bytes[i * 4..i * 4 + 4].copy_from_slice(&word.to_ne_bytes());
        }
        let v6 = Ipv6Addr::from(bytes);
        // Show v4-mapped addresses (::ffff:a.b.c.d) as plain IPv4.
        match v6.to_ipv4_mapped() {
            Some(v4) => IpAddr::V4(v4),
            None => IpAddr::V6(v6),
        }
    } else {
        let word = u32::from_str_radix(addr, 16).ok()?;
        IpAddr::V4(Ipv4Addr::from(word.to_ne_bytes()))
    };
    Some((addr, port))
}

/// Linux `tcp_states.h` numbering (differs from Windows MIB_TCP_STATE).
fn tcp_state_name(st: u8) -> &'static str {
    match st {
        0x01 => "ESTABLISHED",
        0x02 => "SYN_SENT",
        0x03 => "SYN_RCVD",
        0x04 => "FIN_WAIT1",
        0x05 => "FIN_WAIT2",
        0x06 => "TIME_WAIT",
        0x07 => "CLOSED",
        0x08 => "CLOSE_WAIT",
        0x09 => "LAST_ACK",
        0x0A => "LISTEN",
        0x0B => "CLOSING",
        _ => "UNKNOWN",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn endpoints() {
        assert_eq!(
            parse_endpoint("0100007F:0BB8", false),
            Some((IpAddr::V4(Ipv4Addr::LOCALHOST), 3000))
        );
        assert_eq!(
            parse_endpoint("00000000000000000000000001000000:1F90", true),
            Some((IpAddr::V6(Ipv6Addr::LOCALHOST), 8080))
        );
        assert_eq!(
            parse_endpoint("0000000000000000FFFF00000100007F:0050", true),
            Some((IpAddr::V4(Ipv4Addr::LOCALHOST), 80))
        );
    }

    #[test]
    fn line() {
        let owners = HashMap::from([(123456u64, 42u32)]);
        let l = "   0: 0100007F:0BB8 00000000:0000 0A 00000000:00000000 00:00000000 00000000  1000        0 123456 1 0000000000000000 100 0 0 10 0";
        let s = parse_line(l, Protocol::Tcp, false, &owners).unwrap();
        assert_eq!((s.local_port, s.state, s.pid), (3000, Some("LISTEN"), 42));
        assert!(s.remote_addr.is_none());
        assert_eq!(parse_socket_link("socket:[123456]"), Some(123456));
        assert_eq!(parse_socket_link("pipe:[1]"), None);
    }
}
