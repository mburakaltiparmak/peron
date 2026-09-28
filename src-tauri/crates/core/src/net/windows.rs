//! Windows socket tables from the IP Helper API. Buffers are sized with a two-phase call
//! (ERROR_INSUFFICIENT_BUFFER); ports are in network byte order.

use std::ffi::c_void;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

use windows::Win32::Foundation::{ERROR_INSUFFICIENT_BUFFER, NO_ERROR};
use windows::Win32::NetworkManagement::IpHelper::{
    GetExtendedTcpTable, GetExtendedUdpTable, MIB_TCP6TABLE_OWNER_MODULE,
    MIB_TCPTABLE_OWNER_MODULE, MIB_UDP6TABLE_OWNER_MODULE, MIB_UDPTABLE_OWNER_MODULE,
    TCP_TABLE_OWNER_MODULE_ALL, UDP_TABLE_OWNER_MODULE,
};
use windows::Win32::Networking::WinSock::{AF_INET, AF_INET6};

use super::ports::{Protocol, RawSocket};
use crate::error::{AppError, AppResult};

/// Every TCP (all states) and UDP socket for IPv4 and IPv6, with `liCreateTimestamp` open times.
pub fn scan() -> AppResult<Vec<RawSocket>> {
    let mut out = Vec::new();
    tcp4(&mut out)?;
    tcp6(&mut out)?;
    udp4(&mut out)?;
    udp6(&mut out)?;
    Ok(out)
}

/// Two-phase buffer sizing. The buffer is `u64`-backed so row structs are correctly aligned.
fn fetch_table(mut call: impl FnMut(Option<*mut c_void>, &mut u32) -> u32) -> AppResult<Vec<u64>> {
    let mut size = 0u32;
    let mut buf: Vec<u64> = Vec::new();
    for _ in 0..4 {
        let ptr = if buf.is_empty() {
            None
        } else {
            Some(buf.as_mut_ptr() as *mut c_void)
        };
        let ret = call(ptr, &mut size);
        if ret == NO_ERROR.0 && !buf.is_empty() {
            return Ok(buf);
        }
        if ret == ERROR_INSUFFICIENT_BUFFER.0 || (ret == NO_ERROR.0 && buf.is_empty()) {
            // Slack: the table can grow between the two calls.
            let words = (size as usize + 4096) / 8 + 1;
            buf = vec![0u64; words];
            size = (words * 8) as u32;
            continue;
        }
        return Err(AppError::Other(format!(
            "Port tablosu okunamadı (hata {ret})."
        )));
    }
    Err(AppError::Other(
        "Port tablosu okunamadı (tablo sürekli değişiyor).".into(),
    ))
}

/// Reinterprets a table buffer as its rows.
///
/// SAFETY (caller): `buf` must hold a table of type `$table` filled by the matching API call.
macro_rules! rows {
    ($buf:expr, $table:ty) => {{
        let table = &*($buf.as_ptr() as *const $table);
        std::slice::from_raw_parts(table.table.as_ptr(), table.dwNumEntries as usize)
    }};
}

fn tcp4(out: &mut Vec<RawSocket>) -> AppResult<()> {
    // SAFETY: the buffer pointer/size come from `fetch_table` and stay valid for the call.
    let buf = fetch_table(|p, size| unsafe {
        GetExtendedTcpTable(
            p,
            size,
            false,
            AF_INET.0 as u32,
            TCP_TABLE_OWNER_MODULE_ALL,
            0,
        )
    })?;
    // SAFETY: filled by GetExtendedTcpTable with AF_INET + *_OWNER_MODULE_* class.
    for r in unsafe { rows!(buf, MIB_TCPTABLE_OWNER_MODULE) } {
        let state = tcp_state_name(r.dwState);
        let listening = state == "LISTEN";
        out.push(RawSocket {
            protocol: Protocol::Tcp,
            local_addr: IpAddr::V4(ipv4_from_dw(r.dwLocalAddr)),
            local_port: port_from_dw(r.dwLocalPort),
            remote_addr: (!listening).then(|| IpAddr::V4(ipv4_from_dw(r.dwRemoteAddr))),
            remote_port: (!listening).then(|| port_from_dw(r.dwRemotePort)),
            state: Some(state),
            pid: r.dwOwningPid,
            created_at_ms: filetime_to_unix_ms(r.liCreateTimestamp),
        });
    }
    Ok(())
}

fn tcp6(out: &mut Vec<RawSocket>) -> AppResult<()> {
    // SAFETY: see tcp4.
    let buf = fetch_table(|p, size| unsafe {
        GetExtendedTcpTable(
            p,
            size,
            false,
            AF_INET6.0 as u32,
            TCP_TABLE_OWNER_MODULE_ALL,
            0,
        )
    })?;
    // SAFETY: filled by GetExtendedTcpTable with AF_INET6 + *_OWNER_MODULE_* class.
    for r in unsafe { rows!(buf, MIB_TCP6TABLE_OWNER_MODULE) } {
        let state = tcp_state_name(r.dwState);
        let listening = state == "LISTEN";
        out.push(RawSocket {
            protocol: Protocol::Tcp,
            local_addr: IpAddr::V6(Ipv6Addr::from(r.ucLocalAddr)),
            local_port: port_from_dw(r.dwLocalPort),
            remote_addr: (!listening).then(|| IpAddr::V6(Ipv6Addr::from(r.ucRemoteAddr))),
            remote_port: (!listening).then(|| port_from_dw(r.dwRemotePort)),
            state: Some(state),
            pid: r.dwOwningPid,
            created_at_ms: filetime_to_unix_ms(r.liCreateTimestamp),
        });
    }
    Ok(())
}

fn udp4(out: &mut Vec<RawSocket>) -> AppResult<()> {
    // SAFETY: see tcp4.
    let buf = fetch_table(|p, size| unsafe {
        GetExtendedUdpTable(p, size, false, AF_INET.0 as u32, UDP_TABLE_OWNER_MODULE, 0)
    })?;
    // SAFETY: filled by GetExtendedUdpTable with AF_INET + UDP_TABLE_OWNER_MODULE.
    for r in unsafe { rows!(buf, MIB_UDPTABLE_OWNER_MODULE) } {
        out.push(RawSocket {
            protocol: Protocol::Udp,
            local_addr: IpAddr::V4(ipv4_from_dw(r.dwLocalAddr)),
            local_port: port_from_dw(r.dwLocalPort),
            remote_addr: None,
            remote_port: None,
            state: None,
            pid: r.dwOwningPid,
            created_at_ms: filetime_to_unix_ms(r.liCreateTimestamp),
        });
    }
    Ok(())
}

fn udp6(out: &mut Vec<RawSocket>) -> AppResult<()> {
    // SAFETY: see tcp4.
    let buf = fetch_table(|p, size| unsafe {
        GetExtendedUdpTable(p, size, false, AF_INET6.0 as u32, UDP_TABLE_OWNER_MODULE, 0)
    })?;
    // SAFETY: filled by GetExtendedUdpTable with AF_INET6 + UDP_TABLE_OWNER_MODULE.
    for r in unsafe { rows!(buf, MIB_UDP6TABLE_OWNER_MODULE) } {
        out.push(RawSocket {
            protocol: Protocol::Udp,
            local_addr: IpAddr::V6(Ipv6Addr::from(r.ucLocalAddr)),
            local_port: port_from_dw(r.dwLocalPort),
            remote_addr: None,
            remote_port: None,
            state: None,
            pid: r.dwOwningPid,
            created_at_ms: filetime_to_unix_ms(r.liCreateTimestamp),
        });
    }
    Ok(())
}

/// Ports are stored in network byte order in the low 16 bits.
pub fn port_from_dw(dw: u32) -> u16 {
    u16::from_be((dw & 0xFFFF) as u16)
}

/// IPv4 addresses are stored in network byte order.
pub fn ipv4_from_dw(dw: u32) -> Ipv4Addr {
    Ipv4Addr::from(u32::from_be(dw))
}

/// FILETIME (100 ns ticks since 1601-01-01) → Unix milliseconds. `0` means "not recorded".
pub fn filetime_to_unix_ms(ft: i64) -> Option<i64> {
    const EPOCH_DIFF_MS: i64 = 11_644_473_600_000;
    (ft > 0).then(|| ft / 10_000 - EPOCH_DIFF_MS)
}

pub fn tcp_state_name(state: u32) -> &'static str {
    match state {
        1 => "CLOSED",
        2 => "LISTEN",
        3 => "SYN_SENT",
        4 => "SYN_RCVD",
        5 => "ESTABLISHED",
        6 => "FIN_WAIT1",
        7 => "FIN_WAIT2",
        8 => "CLOSE_WAIT",
        9 => "CLOSING",
        10 => "LAST_ACK",
        11 => "TIME_WAIT",
        12 => "DELETE_TCB",
        _ => "UNKNOWN",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn port_is_network_order() {
        // Port 3000 = 0x0BB8, bytes in memory [0x0B, 0xB8, 0, 0] → little-endian u32 0x0000_B80B.
        assert_eq!(port_from_dw(u32::from_le_bytes([0x0B, 0xB8, 0, 0])), 3000);
        assert_eq!(
            port_from_dw(u32::from_le_bytes([0x1F, 0x90, 0xAB, 0xCD])),
            8080
        );
    }

    #[test]
    fn ipv4_is_network_order() {
        assert_eq!(
            ipv4_from_dw(u32::from_le_bytes([127, 0, 0, 1])),
            Ipv4Addr::LOCALHOST
        );
        assert_eq!(
            ipv4_from_dw(u32::from_le_bytes([192, 168, 1, 20])),
            Ipv4Addr::new(192, 168, 1, 20)
        );
    }

    #[test]
    fn filetime_conversion() {
        assert_eq!(filetime_to_unix_ms(0), None);
        assert_eq!(filetime_to_unix_ms(116_444_736_000_000_000), Some(0));
        // 2024-01-01T00:00:00Z
        assert_eq!(
            filetime_to_unix_ms(133_485_408_000_000_000),
            Some(1_704_067_200_000)
        );
    }
}
