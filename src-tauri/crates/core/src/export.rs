//! Snapshot export shared by the desktop app, `peron-cli` and the SSH remote view.
//!
//! - JSON: [`Snapshot`] schema v1 — also the wire format the GUI reads from `peron-cli list --json`
//!   over SSH, so field changes must stay backward compatible (add fields with `#[serde(default)]`).
//! - TXT: aligned, human-readable table in the chosen language, with a header line.
//! - CSV: one row per socket, English column names (for spreadsheets / scripts).

use serde::{Deserialize, Serialize};

use crate::i18n::{self, Lang};
use crate::model::PortEntry;
use crate::util::utc_timestamp;

pub const SCHEMA_VERSION: u32 = 1;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Snapshot {
    pub schema_version: u32,
    /// "peron" (desktop app) or "peron-cli".
    pub tool: String,
    pub version: String,
    pub host: String,
    pub os: String,
    pub generated_at_ms: i64,
    /// False → owners of other users'/elevated processes may be missing.
    pub elevated: bool,
    pub entries: Vec<PortEntry>,
}

impl Snapshot {
    pub fn new(
        tool: &str,
        version: &str,
        elevated: bool,
        entries: Vec<PortEntry>,
        now_ms: i64,
    ) -> Self {
        Snapshot {
            schema_version: SCHEMA_VERSION,
            tool: tool.into(),
            version: version.into(),
            host: host_name(),
            os: crate::platform::os_description(),
            generated_at_ms: now_ms,
            elevated,
            entries,
        }
    }
}

pub fn host_name() -> String {
    sysinfo::System::host_name().unwrap_or_else(|| "localhost".into())
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Format {
    Txt,
    Csv,
    Json,
}

impl Format {
    pub fn parse(s: &str) -> Option<Format> {
        match s.to_ascii_lowercase().as_str() {
            "txt" | "text" => Some(Format::Txt),
            "csv" => Some(Format::Csv),
            "json" => Some(Format::Json),
            _ => None,
        }
    }

    pub fn extension(self) -> &'static str {
        match self {
            Format::Txt => "txt",
            Format::Csv => "csv",
            Format::Json => "json",
        }
    }
}

/// `peron-web-01-2026-09-25T19-15-04Z.txt` (UTC; `:` is not allowed in Windows file names).
pub fn file_name(prefix: &str, host: &str, at_ms: i64, format: Format) -> String {
    let host: String = host
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' || c == '.' {
                c
            } else {
                '_'
            }
        })
        .collect();
    format!(
        "{prefix}-{host}-{}.{}",
        utc_timestamp(at_ms).replace(':', "-"),
        format.extension()
    )
}

pub fn render(s: &Snapshot, format: Format, lang: Lang) -> String {
    match format {
        Format::Json => serde_json::to_string_pretty(s).unwrap_or_default(),
        Format::Csv => csv(s),
        Format::Txt => txt(s, lang),
    }
}

fn address(e: &PortEntry) -> String {
    match (&e.remote_addr, e.remote_port) {
        (Some(r), Some(p)) => format!("{}:{} → {r}:{p}", e.local_addr, e.local_port),
        _ => e.local_addr.clone(),
    }
}

pub fn human_bytes(b: u64) -> String {
    const UNITS: [&str; 4] = ["KB", "MB", "GB", "TB"];
    if b < 1024 {
        return format!("{b} B");
    }
    let mut v = b as f64 / 1024.0;
    let mut i = 0;
    while v >= 1024.0 && i < UNITS.len() - 1 {
        v /= 1024.0;
        i += 1;
    }
    if v < 10.0 {
        format!("{v:.1} {}", UNITS[i])
    } else {
        format!("{v:.0} {}", UNITS[i])
    }
}

fn txt(s: &Snapshot, lang: Lang) -> String {
    let (h_title, h_ports, h_partial, cols) = match lang {
        Lang::Tr => (
            "host",
            "soket",
            "Sınırlı yetki: bazı süreçlerin sahibi görünmüyor (yönetici/root olarak çalıştırın).",
            [
                "PORT",
                "PROTOKOL",
                "ADRES",
                "DURUM",
                "PID",
                "SÜREÇ",
                "BAŞLATAN",
                "PROJE",
                "AÇIK KALMA",
                "CPU",
                "RAM",
            ],
        ),
        Lang::En => (
            "host",
            "sockets",
            "Limited rights: some owners are hidden (run as administrator/root).",
            [
                "PORT",
                "PROTO",
                "ADDRESS",
                "STATE",
                "PID",
                "PROCESS",
                "STARTED BY",
                "PROJECT",
                "OPEN FOR",
                "CPU",
                "RAM",
            ],
        ),
    };
    let rows: Vec<[String; 11]> = s
        .entries
        .iter()
        .map(|e| {
            [
                e.local_port.to_string(),
                format!("{:?}", e.protocol).to_uppercase(),
                address(e),
                e.state.clone(),
                e.pid.to_string(),
                e.process_name.clone(),
                e.parent_name.clone().unwrap_or_else(|| "—".into()),
                e.project.clone().unwrap_or_else(|| "—".into()),
                e.opened_at_ms
                    .map(|t| i18n::duration(lang, s.generated_at_ms - t))
                    .unwrap_or_else(|| "—".into()),
                format!("{:.1}%", e.cpu_percent),
                if e.memory_bytes > 0 {
                    human_bytes(e.memory_bytes)
                } else {
                    "—".into()
                },
            ]
        })
        .collect();
    let mut widths = cols.map(|c| c.chars().count());
    for r in &rows {
        for (w, cell) in widths.iter_mut().zip(r) {
            *w = (*w).max(cell.chars().count());
        }
    }
    let line = |cells: &[String]| -> String {
        cells
            .iter()
            .zip(widths)
            .map(|(c, w)| format!("{c}{}", " ".repeat(w - c.chars().count())))
            .collect::<Vec<_>>()
            .join("  ")
            .trim_end()
            .to_string()
    };
    let mut out = format!(
        "Peron {} · {h_title}: {} · {} · {}\n{} {h_ports}\n",
        s.version,
        s.host,
        s.os,
        utc_timestamp(s.generated_at_ms),
        s.entries.len()
    );
    if !s.elevated {
        out.push_str(h_partial);
        out.push('\n');
    }
    out.push('\n');
    out.push_str(&line(&cols.map(String::from)));
    out.push('\n');
    for r in &rows {
        out.push_str(&line(r));
        out.push('\n');
    }
    out
}

fn csv_field(v: &str) -> String {
    if v.contains([',', '"', '\n', '\r']) {
        format!("\"{}\"", v.replace('"', "\"\""))
    } else {
        v.to_string()
    }
}

fn csv(s: &Snapshot) -> String {
    let mut out = String::from(
        "host,generated_at,port,protocol,ip_version,local_address,remote_address,remote_port,state,pid,process,parent,project,working_dir,command,opened_at,open_for_seconds,cpu_percent,memory_bytes,is_system,is_dev,exposed\n",
    );
    let at = utc_timestamp(s.generated_at_ms);
    for e in &s.entries {
        let fields = [
            s.host.clone(),
            at.clone(),
            e.local_port.to_string(),
            format!("{:?}", e.protocol).to_uppercase(),
            e.ip_version.to_string(),
            e.local_addr.clone(),
            e.remote_addr.clone().unwrap_or_default(),
            e.remote_port.map(|p| p.to_string()).unwrap_or_default(),
            e.state.clone(),
            e.pid.to_string(),
            e.process_name.clone(),
            e.parent_name.clone().unwrap_or_default(),
            e.project.clone().unwrap_or_default(),
            e.cwd.clone().unwrap_or_default(),
            e.cmdline.clone().unwrap_or_default(),
            e.opened_at_ms.map(utc_timestamp).unwrap_or_default(),
            e.opened_at_ms
                .map(|t| ((s.generated_at_ms - t).max(0) / 1000).to_string())
                .unwrap_or_default(),
            format!("{:.1}", e.cpu_percent),
            e.memory_bytes.to_string(),
            e.is_system.to_string(),
            e.is_dev.to_string(),
            (!e.is_localhost_only && e.is_listening()).to_string(),
        ];
        out.push_str(
            &fields
                .iter()
                .map(|f| csv_field(f))
                .collect::<Vec<_>>()
                .join(","),
        );
        out.push('\n');
    }
    out
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use crate::net::ports::Protocol;

    pub(crate) fn entry(port: u16, name: &str) -> PortEntry {
        PortEntry {
            id: format!("TCP|0.0.0.0|{port}||0|1"),
            protocol: Protocol::Tcp,
            ip_version: 4,
            local_addr: "0.0.0.0".into(),
            local_port: port,
            remote_addr: None,
            remote_port: None,
            state: "LISTEN".into(),
            pid: 4242,
            process_name: name.into(),
            exe_path: None,
            cmdline: Some("node server.js --title \"a, b\"".into()),
            cwd: Some("/srv/shop-api".into()),
            project: Some("shop-api".into()),
            parent_pid: Some(1),
            parent_name: Some("bash".into()),
            opened_at_ms: Some(1_000_000 - 90 * 60_000),
            opened_at_source: "socket".into(),
            process_start_ms: 1_000_000 - 90 * 60_000,
            cpu_percent: 1.25,
            memory_bytes: 50 * 1024 * 1024,
            disk_read_bytes: 0,
            disk_written_bytes: 0,
            is_system: false,
            is_protected: false,
            is_dev: true,
            is_localhost_only: false,
            accessible: true,
        }
    }

    pub(crate) fn snap() -> Snapshot {
        Snapshot {
            schema_version: SCHEMA_VERSION,
            tool: "peron-cli".into(),
            version: "0.1.0-beta.1".into(),
            host: "web-01".into(),
            os: "Ubuntu 24.04".into(),
            generated_at_ms: 1_000_000,
            elevated: false,
            entries: vec![entry(3000, "node"), entry(8080, "java")],
        }
    }

    #[test]
    fn json_round_trips() {
        let s = snap();
        let text = render(&s, Format::Json, Lang::En);
        let back: Snapshot = serde_json::from_str(&text).unwrap();
        assert_eq!(back.entries.len(), 2);
        assert_eq!(back.entries[0].project.as_deref(), Some("shop-api"));
        assert!(text.contains("\"schemaVersion\": 1"));
    }

    #[test]
    fn txt_is_aligned_and_localized() {
        let en = render(&snap(), Format::Txt, Lang::En);
        let lines: Vec<&str> = en.lines().collect();
        assert!(lines[0].starts_with("Peron 0.1.0-beta.1 · host: web-01"));
        assert!(en.contains("Limited rights"));
        let header = lines.iter().find(|l| l.starts_with("PORT")).unwrap();
        let row = lines.iter().find(|l| l.starts_with("3000")).unwrap();
        // Columns line up: "PROCESS" header and "node" start at the same offset.
        assert_eq!(header.find("PROCESS"), row.find("node"));
        assert!(row.contains("1 h 30 min"));
        let tr = render(&snap(), Format::Txt, Lang::Tr);
        assert!(tr.contains("SÜREÇ") && tr.contains("1 sa 30 dk"));
    }

    #[test]
    fn csv_quotes_fields() {
        let c = render(&snap(), Format::Csv, Lang::En);
        let row = c.lines().nth(1).unwrap();
        assert!(row.starts_with("web-01,1970-01-01T00:16:40Z,3000,TCP,4,0.0.0.0,"));
        assert!(row.contains("\"node server.js --title \"\"a, b\"\"\""));
        assert!(
            row.ends_with(",5400,1.2,52428800,false,true,true")
                || row.ends_with(",5400,1.3,52428800,false,true,true")
        );
    }

    #[test]
    fn file_names_are_safe() {
        assert_eq!(
            file_name("peron", "web 01/prod", 1_790_359_384_000, Format::Csv),
            "peron-web_01_prod-2026-09-25T18-03-04Z.csv"
        );
    }
}
