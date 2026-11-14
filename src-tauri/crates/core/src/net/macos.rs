//! macOS socket tables via libproc: every process's file descriptors (`PROC_PIDLISTFDS`), then
//! `PROC_PIDFDSOCKETINFO` for the socket ones.
//!
//! Without root, other users' processes (e.g. root-owned system daemons) can't be inspected, so
//! their sockets are missing from the list entirely. The kernel exposes no socket creation time,
//! so `created_at_ms` is always `None` and the UI falls back to the process start time.

use std::collections::HashSet;
use std::ffi::c_void;
use std::mem::{offset_of, size_of};
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

use super::ports::{Protocol, RawSocket};
use crate::error::AppResult;

// <sys/proc_info.h> constants not exported by the `libc` crate.
const PROC_PIDFDSOCKETINFO: i32 = 3;
const SOCKINFO_IN: i32 = 1;
const SOCKINFO_TCP: i32 = 2;
const INI_IPV4: u8 = 0x1;
const INI_IPV6: u8 = 0x2;

/// `struct socket_fdinfo` reduced to the fields Peron reads; the rest is opaque padding.
/// Layout checked against the macOS SDK by the assertions below.
#[repr(C)]
struct SocketFdInfo {
    _pfi: [u8; 24],       // struct proc_fileinfo
    _soi_stat: [u8; 136], // struct vinfo_stat
    soi_so: u64,          // kernel socket address (permuted, but unique per socket)
    _soi_pcb: u64,
    _soi_type: i32,
    soi_protocol: i32,
    soi_family: i32,
    _soi_misc: [u8; 68], // options … soi_snd
    soi_kind: i32,
    _rfu_1: u32,
    proto: InTcpSockInfo,
}

/// `struct tcp_sockinfo`, whose first member is `struct in_sockinfo`; UDP sockets (`SOCKINFO_IN`)
/// fill only that first part.
#[repr(C)]
struct InTcpSockInfo {
    /// Ports are `int`s holding a network-order `u16`.
    insi_fport: i32,
    insi_lport: i32,
    _gencnt_flags_flow: [u8; 16],
    insi_vflag: u8,
    _ttl_rfu: [u8; 7],
    /// `union { struct in4in6_addr; struct in6_addr }`: IPv4 lives in the last 4 bytes.
    insi_faddr: [u8; 16],
    insi_laddr: [u8; 16],
    _v4_v6: [u8; 16],
    tcpsi_state: i32,
    _rest: [u8; 444], // timers … end of the soi_proto union
}

const _: () = {
    assert!(size_of::<SocketFdInfo>() == 792);
    assert!(offset_of!(SocketFdInfo, soi_so) == 24 + 136);
    assert!(offset_of!(SocketFdInfo, soi_family) == 24 + 160);
    assert!(offset_of!(SocketFdInfo, soi_kind) == 24 + 232);
    assert!(offset_of!(SocketFdInfo, proto) == 24 + 240);
    assert!(offset_of!(InTcpSockInfo, insi_vflag) == 24);
    assert!(offset_of!(InTcpSockInfo, insi_faddr) == 32);
    assert!(offset_of!(InTcpSockInfo, insi_laddr) == 48);
    assert!(offset_of!(InTcpSockInfo, tcpsi_state) == 80);
};

pub fn scan() -> AppResult<Vec<RawSocket>> {
    let mut seen = HashSet::new();
    let mut out = Vec::new();
    for pid in all_pids() {
        for fd in socket_fds(pid) {
            let Some(info) = socket_info(pid, fd) else {
                continue;
            };
            // A forked child shares its parent's sockets; list each socket once (first pid wins).
            if info.soi_so != 0 && !seen.insert(info.soi_so) {
                continue;
            }
            if let Some(s) = to_raw(&info, pid as u32) {
                out.push(s);
            }
        }
    }
    Ok(out)
}

fn all_pids() -> Vec<libc::pid_t> {
    // SAFETY: a null buffer only asks for the current process count.
    let n = unsafe { libc::proc_listallpids(std::ptr::null_mut(), 0) };
    if n <= 0 {
        return Vec::new();
    }
    // Head-room for processes started between the two calls.
    let mut pids = vec![0 as libc::pid_t; n as usize + 64];
    let bytes = (pids.len() * size_of::<libc::pid_t>()) as i32;
    // SAFETY: the buffer is `bytes` long and only written by the kernel.
    let n = unsafe { libc::proc_listallpids(pids.as_mut_ptr().cast::<c_void>(), bytes) };
    pids.truncate(n.max(0) as usize);
    pids.retain(|&p| p > 0);
    pids
}

/// Socket file descriptors of `pid`; empty when the process is gone or not ours to inspect.
fn socket_fds(pid: libc::pid_t) -> Vec<i32> {
    let size = size_of::<libc::proc_fdinfo>();
    // SAFETY: a null buffer only asks for the required size in bytes.
    let needed =
        unsafe { libc::proc_pidinfo(pid, libc::PROC_PIDLISTFDS, 0, std::ptr::null_mut(), 0) };
    if needed <= 0 {
        return Vec::new();
    }
    let mut fds: Vec<libc::proc_fdinfo> = Vec::with_capacity(needed as usize / size + 16);
    let bytes = (fds.capacity() * size) as i32;
    // SAFETY: the kernel writes at most `bytes` into the spare capacity and returns how many it
    // wrote; `proc_fdinfo` is plain integers, so any written prefix is valid.
    let written = unsafe {
        libc::proc_pidinfo(
            pid,
            libc::PROC_PIDLISTFDS,
            0,
            fds.as_mut_ptr().cast::<c_void>(),
            bytes,
        )
    };
    if written <= 0 {
        return Vec::new();
    }
    // SAFETY: `written / size` entries were initialised above, within capacity.
    unsafe { fds.set_len((written as usize / size).min(fds.capacity())) };
    fds.iter()
        .filter(|f| f.proc_fdtype == libc::PROX_FDTYPE_SOCKET as u32)
        .map(|f| f.proc_fd)
        .collect()
}

fn socket_info(pid: libc::pid_t, fd: i32) -> Option<SocketFdInfo> {
    let mut info = std::mem::MaybeUninit::<SocketFdInfo>::zeroed();
    let size = size_of::<SocketFdInfo>() as i32;
    // SAFETY: the buffer is exactly one `socket_fdinfo`; all-zero bytes are a valid value.
    let n = unsafe {
        libc::proc_pidfdinfo(
            pid,
            fd,
            PROC_PIDFDSOCKETINFO,
            info.as_mut_ptr().cast::<c_void>(),
            size,
        )
    };
    // SAFETY: zero-initialised, and the kernel only writes plain integer fields.
    (n == size).then(|| unsafe { info.assume_init() })
}

fn to_raw(info: &SocketFdInfo, pid: u32) -> Option<RawSocket> {
    if info.soi_family != libc::AF_INET && info.soi_family != libc::AF_INET6 {
        return None;
    }
    let protocol = match info.soi_kind {
        SOCKINFO_TCP => Protocol::Tcp,
        SOCKINFO_IN if info.soi_protocol == libc::IPPROTO_UDP => Protocol::Udp,
        _ => return None,
    };
    let p = &info.proto;
    let v6 = p.insi_vflag & INI_IPV6 != 0;
    if !v6 && p.insi_vflag & INI_IPV4 == 0 {
        return None;
    }
    let (state, listening) = match protocol {
        Protocol::Tcp => {
            let s = tcp_state_name(p.tcpsi_state);
            (Some(s), s == "LISTEN")
        }
        Protocol::Udp => (None, true),
    };
    Some(RawSocket {
        protocol,
        local_addr: addr(&p.insi_laddr, v6),
        local_port: port(p.insi_lport),
        remote_addr: (!listening).then(|| addr(&p.insi_faddr, v6)),
        remote_port: (!listening).then(|| port(p.insi_fport)),
        state,
        pid,
        created_at_ms: None,
    })
}

fn port(raw: i32) -> u16 {
    u16::from_be(raw as u16)
}

fn addr(bytes: &[u8; 16], v6: bool) -> IpAddr {
    if v6 {
        let a = Ipv6Addr::from(*bytes);
        // Show v4-mapped addresses (::ffff:a.b.c.d) as plain IPv4, as on Windows/Linux.
        match a.to_ipv4_mapped() {
            Some(v4) => IpAddr::V4(v4),
            None => IpAddr::V6(a),
        }
    } else {
        IpAddr::V4(Ipv4Addr::new(bytes[12], bytes[13], bytes[14], bytes[15]))
    }
}

/// `TSI_S_*` numbering from <sys/proc_info.h> (BSD order; differs from Linux and Windows).
fn tcp_state_name(st: i32) -> &'static str {
    match st {
        0 => "CLOSED",
        1 => "LISTEN",
        2 => "SYN_SENT",
        3 => "SYN_RCVD",
        4 => "ESTABLISHED",
        5 => "CLOSE_WAIT",
        6 => "FIN_WAIT1",
        7 => "CLOSING",
        8 => "LAST_ACK",
        9 => "FIN_WAIT2",
        10 => "TIME_WAIT",
        _ => "UNKNOWN",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn endpoints() {
        assert_eq!(port(u16::to_be(3000) as i32), 3000);
        let mut v4 = [0u8; 16];
        v4[12..].copy_from_slice(&[127, 0, 0, 1]);
        assert_eq!(addr(&v4, false), IpAddr::V4(Ipv4Addr::LOCALHOST));
        assert_eq!(
            addr(&Ipv6Addr::LOCALHOST.octets(), true),
            IpAddr::V6(Ipv6Addr::LOCALHOST)
        );
        let mapped = Ipv4Addr::LOCALHOST.to_ipv6_mapped().octets();
        assert_eq!(addr(&mapped, true), IpAddr::V4(Ipv4Addr::LOCALHOST));
        assert_eq!(tcp_state_name(1), "LISTEN");
        assert_eq!(tcp_state_name(4), "ESTABLISHED");
    }

    #[test]
    fn sees_own_udp_and_tcp_v6() {
        let udp = std::net::UdpSocket::bind("127.0.0.1:0").expect("udp");
        let tcp6 = std::net::TcpListener::bind("[::1]:0").expect("tcp6");
        let (up, tp) = (
            udp.local_addr().unwrap().port(),
            tcp6.local_addr().unwrap().port(),
        );
        let all = scan().expect("scan");
        let me = std::process::id();
        assert!(all
            .iter()
            .any(|s| s.protocol == Protocol::Udp && s.local_port == up && s.pid == me));
        let t = all
            .iter()
            .find(|s| s.protocol == Protocol::Tcp && s.local_port == tp)
            .expect("v6 listener");
        assert_eq!(t.local_addr, IpAddr::V6(Ipv6Addr::LOCALHOST));
        assert_eq!(t.state, Some("LISTEN"));
    }
}
