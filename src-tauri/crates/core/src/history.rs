//! Port open/close events, derived by diffing two snapshots. Shared by the desktop app (History
//! tab, "exposed port" alerts) and `peron-cli watch`.
//!
//! Only listening TCP sockets are tracked (that's what "a port is open" means to users). The IPv4
//! and IPv6 sockets of one process on one port are a single listener. A listener is identified by
//! (pid, port, process start time), so a restarted server is "closed" + "opened".

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::i18n::Lang;
use crate::model::PortEntry;
use crate::util::utc_timestamp;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum EventKind {
    Opened,
    Closed,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PortEvent {
    pub at_ms: i64,
    pub kind: EventKind,
    pub port: u16,
    pub pid: u32,
    pub process: String,
    #[serde(default)]
    pub project: Option<String>,
    /// Local addresses of the listener, e.g. ["0.0.0.0", "[::]"].
    pub addrs: Vec<String>,
    /// Reachable from other machines (bound to a non-loopback address).
    pub exposed: bool,
    pub is_system: bool,
    pub is_dev: bool,
}

type Key = (u32, u16, i64);

fn listeners(entries: &[PortEntry]) -> BTreeMap<Key, Vec<&PortEntry>> {
    let mut map: BTreeMap<Key, Vec<&PortEntry>> = BTreeMap::new();
    for e in entries.iter().filter(|e| e.is_listening()) {
        map.entry((e.pid, e.local_port, e.process_start_ms))
            .or_default()
            .push(e);
    }
    map
}

fn event(kind: EventKind, at_ms: i64, sockets: &[&PortEntry]) -> PortEvent {
    let first = sockets[0];
    let mut addrs: Vec<String> = sockets.iter().map(|e| e.local_addr.clone()).collect();
    addrs.dedup();
    PortEvent {
        at_ms,
        kind,
        port: first.local_port,
        pid: first.pid,
        process: first.process_name.clone(),
        project: sockets.iter().find_map(|e| e.project.clone()),
        exposed: sockets.iter().any(|e| !e.is_localhost_only),
        addrs,
        is_system: first.is_system,
        is_dev: first.is_dev,
    }
}

/// Events between two snapshots, closed before opened, each group sorted by port.
pub fn diff(prev: &[PortEntry], next: &[PortEntry], at_ms: i64) -> Vec<PortEvent> {
    let (before, after) = (listeners(prev), listeners(next));
    let mut out: Vec<PortEvent> = before
        .iter()
        .filter(|(k, _)| !after.contains_key(k))
        .map(|(_, s)| event(EventKind::Closed, at_ms, s))
        .collect();
    out.extend(
        after
            .iter()
            .filter(|(k, _)| !before.contains_key(k))
            .map(|(_, s)| event(EventKind::Opened, at_ms, s)),
    );
    out.sort_by_key(|e| (e.kind == EventKind::Opened, e.port));
    out
}

/// One human-readable line (CLI `watch`, TXT export):
/// `2026-09-25T19:15:04Z  OPENED  3000/tcp  node (4242)  shop-api  0.0.0.0, [::]  EXPOSED`
pub fn line(e: &PortEvent, lang: Lang) -> String {
    let (kind, exposed) = match (lang, e.kind) {
        (Lang::Tr, EventKind::Opened) => ("AÇILDI  ", "AĞA AÇIK"),
        (Lang::Tr, EventKind::Closed) => ("KAPANDI ", "AĞA AÇIK"),
        (Lang::En, EventKind::Opened) => ("OPENED  ", "EXPOSED"),
        (Lang::En, EventKind::Closed) => ("CLOSED  ", "EXPOSED"),
    };
    let mut s = format!(
        "{}  {kind}{:>5}/tcp  {} ({})  {}  {}",
        utc_timestamp(e.at_ms),
        e.port,
        e.process,
        e.pid,
        e.project.as_deref().unwrap_or("—"),
        e.addrs.join(", ")
    );
    if e.exposed && e.kind == EventKind::Opened {
        s.push_str("  ");
        s.push_str(exposed);
    }
    s
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct HistoryDoc<'a> {
    schema_version: u32,
    host: &'a str,
    generated_at_ms: i64,
    events: &'a [PortEvent],
}

/// Export of the event history in the same three formats as port snapshots.
pub fn render(
    events: &[PortEvent],
    format: crate::export::Format,
    lang: Lang,
    host: &str,
    now_ms: i64,
) -> String {
    use crate::export::Format;
    match format {
        Format::Json => serde_json::to_string_pretty(&HistoryDoc {
            schema_version: crate::export::SCHEMA_VERSION,
            host,
            generated_at_ms: now_ms,
            events,
        })
        .unwrap_or_default(),
        Format::Txt => {
            let title = match lang {
                Lang::Tr => "olay",
                Lang::En => "events",
            };
            let mut out = format!(
                "Peron · host: {host} · {} · {} {title}\n\n",
                utc_timestamp(now_ms),
                events.len()
            );
            for e in events {
                out.push_str(&line(e, lang));
                out.push('\n');
            }
            out
        }
        Format::Csv => {
            let mut out = String::from(
                "time,event,port,pid,process,project,addresses,exposed,is_system,is_dev\n",
            );
            for e in events {
                let addrs = e.addrs.join(" ");
                out.push_str(&format!(
                    "{},{},{},{},{},{},{},{},{},{}\n",
                    utc_timestamp(e.at_ms),
                    match e.kind {
                        EventKind::Opened => "opened",
                        EventKind::Closed => "closed",
                    },
                    e.port,
                    e.pid,
                    csv_safe(&e.process),
                    csv_safe(e.project.as_deref().unwrap_or("")),
                    csv_safe(&addrs),
                    e.exposed,
                    e.is_system,
                    e.is_dev,
                ));
            }
            out
        }
    }
}

fn csv_safe(v: &str) -> String {
    if v.contains([',', '"', '\n']) {
        format!("\"{}\"", v.replace('"', "\"\""))
    } else {
        v.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::export::tests::entry;

    fn listener(port: u16, pid: u32, addr: &str, local_only: bool) -> PortEntry {
        let mut e = entry(port, "node");
        e.pid = pid;
        e.local_addr = addr.into();
        e.is_localhost_only = local_only;
        e
    }

    #[test]
    fn opened_and_closed() {
        let a = vec![
            listener(3000, 1, "127.0.0.1", true),
            listener(5173, 2, "127.0.0.1", true),
        ];
        let b = vec![
            listener(5173, 2, "127.0.0.1", true),
            listener(8080, 3, "0.0.0.0", false),
            listener(8080, 3, "[::]", false),
        ];
        let ev = diff(&a, &b, 42);
        assert_eq!(ev.len(), 2);
        assert_eq!((ev[0].kind, ev[0].port), (EventKind::Closed, 3000));
        assert_eq!((ev[1].kind, ev[1].port), (EventKind::Opened, 8080));
        assert!(ev[1].exposed);
        assert_eq!(ev[1].addrs, vec!["0.0.0.0", "[::]"]);
    }

    #[test]
    fn restart_is_close_plus_open_and_non_listeners_ignored() {
        let mut restarted = listener(3000, 1, "127.0.0.1", true);
        restarted.process_start_ms += 5_000;
        let mut established = listener(3000, 1, "127.0.0.1", true);
        established.state = "ESTABLISHED".into();
        let ev = diff(
            &[listener(3000, 1, "127.0.0.1", true)],
            &[restarted, established],
            1,
        );
        assert_eq!(
            ev.iter().map(|e| e.kind).collect::<Vec<_>>(),
            vec![EventKind::Closed, EventKind::Opened]
        );
        assert!(diff(&[], &[], 1).is_empty());
    }

    #[test]
    fn lines_and_exports() {
        let ev = diff(
            &[],
            &[listener(8080, 3, "0.0.0.0", false)],
            1_790_359_384_000,
        );
        let l = line(&ev[0], Lang::En);
        assert!(l.starts_with("2026-09-25T18:03:04Z  OPENED"));
        assert!(l.contains(" 8080/tcp  node (3)  shop-api  0.0.0.0  EXPOSED"));
        assert!(line(&ev[0], Lang::Tr).contains("AĞA AÇIK"));
        let csv = render(&ev, crate::export::Format::Csv, Lang::En, "web-01", 0);
        assert!(csv
            .lines()
            .nth(1)
            .unwrap()
            .starts_with("2026-09-25T18:03:04Z,opened,8080,3,node,shop-api,0.0.0.0,true"));
        let json = render(&ev, crate::export::Format::Json, Lang::En, "web-01", 0);
        assert!(json.contains("\"kind\": \"opened\""));
    }
}
