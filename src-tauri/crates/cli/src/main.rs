//! peron-cli — Peron for terminals and headless servers (and the desktop app's SSH remote view).
//!
//! Exit codes: 0 ok · 1 error · 2 usage · 3 not found · 4 access denied · 5 process changed (PID
//! reused) · 6 protected process. The remote view relies on these codes and on `list --format json`
//! (schema: peron_core::export::Snapshot) — keep both stable.

use std::io::{BufRead, IsTerminal, Write};
use std::process::ExitCode;
use std::time::Duration;

use peron_core::error::AppError;
use peron_core::export::{self, Format, Snapshot};
use peron_core::history;
use peron_core::i18n::Lang;
use peron_core::model::{self, PortEntry};
use peron_core::net::ports::Protocol;
use peron_core::proc::info::Detail;
use peron_core::proc::{elevate, kill};
use peron_core::util::now_ms;
use sysinfo::System;

const VERSION: &str = env!("CARGO_PKG_VERSION");

const HELP: &str = "\
peron-cli — list, export, watch and close open ports (https://burakaltiparmak.dev/products/peron)

USAGE:
  peron-cli list   [--all] [--udp] [--system] [--format txt|csv|json] [--lang tr|en]
  peron-cli export [--all] [--udp] [--system] [--format txt|csv|json] [--output FILE|-]
  peron-cli watch  [--interval SECS] [--system] [--format txt|json] [--lang tr|en]
  peron-cli kill   PORT [--yes] [--tree]
  peron-cli kill   --pid PID --start-ms MS [--yes] [--tree]
  peron-cli --version | --help

By default only listening TCP ports of non-system processes are shown.
  --all       every TCP connection (ESTABLISHED, TIME_WAIT, …)
  --udp       include UDP sockets
  --system    include system processes
  --json      shorthand for --format json
kill asks for confirmation on a terminal; in scripts pass --yes. Run as root/administrator to see
and manage processes of other users.

Exit codes: 0 ok, 1 error, 2 usage, 3 not found, 4 access denied, 5 process changed, 6 protected";

#[derive(Debug, Default, PartialEq)]
struct Args {
    command: String,
    all: bool,
    udp: bool,
    system: bool,
    format: Option<Format>,
    lang: Option<Lang>,
    output: Option<String>,
    interval: Option<u64>,
    yes: bool,
    tree: bool,
    port: Option<u16>,
    pid: Option<u32>,
    start_ms: Option<i64>,
}

fn parse(argv: &[String]) -> Result<Args, String> {
    let mut a = Args::default();
    let mut it = argv.iter();
    let value = |name: &str, it: &mut std::slice::Iter<String>| -> Result<String, String> {
        it.next()
            .cloned()
            .ok_or_else(|| format!("{name} needs a value"))
    };
    while let Some(arg) = it.next() {
        match arg.as_str() {
            "--all" | "-a" => a.all = true,
            "--udp" | "-u" => a.udp = true,
            "--system" | "-s" => a.system = true,
            "--yes" | "-y" => a.yes = true,
            "--tree" => a.tree = true,
            "--json" => a.format = Some(Format::Json),
            "--format" | "-f" => {
                let v = value(arg, &mut it)?;
                a.format = Some(Format::parse(&v).ok_or(format!("unknown format: {v}"))?);
            }
            "--lang" => {
                a.lang = Some(match value(arg, &mut it)?.as_str() {
                    "tr" => Lang::Tr,
                    "en" => Lang::En,
                    v => return Err(format!("unknown language: {v}")),
                })
            }
            "--output" | "-o" => a.output = Some(value(arg, &mut it)?),
            "--interval" | "-i" => {
                let v = value(arg, &mut it)?;
                a.interval = Some(
                    v.parse()
                        .ok()
                        .filter(|s| (1..=3600).contains(s))
                        .ok_or(format!("bad interval: {v}"))?,
                );
            }
            "--pid" => {
                let v = value(arg, &mut it)?;
                a.pid = Some(v.parse().map_err(|_| format!("bad PID: {v}"))?);
            }
            "--start-ms" => {
                let v = value(arg, &mut it)?;
                a.start_ms = Some(v.parse().map_err(|_| format!("bad --start-ms: {v}"))?);
            }
            "--help" | "-h" | "help" => a.command = "help".into(),
            "--version" | "-V" | "version" => a.command = "version".into(),
            s if s.starts_with('-') => return Err(format!("unknown option: {s}")),
            s if a.command.is_empty() => a.command = s.to_string(),
            s if a.command == "kill" && a.port.is_none() => {
                a.port = Some(
                    s.parse()
                        .ok()
                        .filter(|p| *p > 0)
                        .ok_or(format!("bad port: {s}"))?,
                )
            }
            s => return Err(format!("unexpected argument: {s}")),
        }
    }
    if a.command.is_empty() {
        a.command = "list".into();
    }
    Ok(a)
}

fn lang(a: &Args) -> Lang {
    a.lang.unwrap_or_else(|| Lang::resolve(None))
}

/// Same filter semantics as the GUI: listening TCP (+ UDP with --udp), all states with --all,
/// system processes only with --system.
fn keep(e: &PortEntry, a: &Args) -> bool {
    let kind_ok = match e.protocol {
        Protocol::Tcp => a.all || e.state == "LISTEN",
        Protocol::Udp => a.udp,
    };
    kind_ok && (a.system || !e.is_system)
}

/// Two samples ~0.3 s apart so CPU % is meaningful.
fn scan(detail: Detail) -> Result<Vec<PortEntry>, AppError> {
    let mut sys = System::new();
    model::build_snapshot(&mut sys, detail)?;
    if detail == Detail::Full {
        std::thread::sleep(Duration::from_millis(300));
    }
    model::build_snapshot(&mut sys, detail)
}

fn snapshot(a: &Args) -> Result<Snapshot, AppError> {
    let entries = scan(Detail::Full)?
        .into_iter()
        .filter(|e| keep(e, a))
        .collect();
    Ok(Snapshot::new(
        "peron-cli",
        VERSION,
        elevate::is_elevated(),
        entries,
        now_ms(),
    ))
}

fn hint_if_limited(s: &Snapshot) {
    if !s.elevated && s.entries.iter().any(|e| e.pid == 0 || !e.accessible) {
        let root = if cfg!(windows) {
            "an administrator terminal"
        } else {
            "sudo"
        };
        eprintln!("note: some owners are hidden; run with {root} to see all.");
    }
}

fn cmd_list(a: &Args) -> Result<ExitCode, AppError> {
    let s = snapshot(a)?;
    print!(
        "{}",
        export::render(&s, a.format.unwrap_or(Format::Txt), lang(a))
    );
    if a.format.unwrap_or(Format::Txt) != Format::Json {
        hint_if_limited(&s);
    }
    Ok(ExitCode::SUCCESS)
}

fn cmd_export(a: &Args) -> Result<ExitCode, AppError> {
    let s = snapshot(a)?;
    let format = a.format.unwrap_or(Format::Txt);
    let text = export::render(&s, format, lang(a));
    match a.output.as_deref() {
        Some("-") => print!("{text}"),
        out => {
            let path = out
                .map(String::from)
                .unwrap_or_else(|| export::file_name("peron", &s.host, s.generated_at_ms, format));
            std::fs::write(&path, text).map_err(|e| AppError::Other(format!("{path}: {e}")))?;
            println!("{path}");
        }
    }
    Ok(ExitCode::SUCCESS)
}

fn cmd_watch(a: &Args) -> Result<ExitCode, AppError> {
    let interval = Duration::from_secs(a.interval.unwrap_or(2));
    let json = a.format == Some(Format::Json);
    let filter = |v: Vec<PortEntry>| -> Vec<PortEntry> {
        v.into_iter().filter(|e| a.system || !e.is_system).collect()
    };
    let mut prev = filter(scan(Detail::Light)?);
    eprintln!(
        "watching {} listening ports every {}s — Ctrl+C to stop",
        prev.iter().filter(|e| e.is_listening()).count(),
        interval.as_secs()
    );
    let stdout = std::io::stdout();
    loop {
        std::thread::sleep(interval);
        let next = filter(scan(Detail::Light)?);
        let mut out = stdout.lock();
        for e in history::diff(&prev, &next, now_ms()) {
            let line = if json {
                serde_json::to_string(&e).unwrap_or_default()
            } else {
                history::line(&e, lang(a))
            };
            if writeln!(out, "{line}").is_err() {
                return Ok(ExitCode::SUCCESS); // stdout closed (e.g. `| head`)
            }
        }
        let _ = out.flush();
        prev = next;
    }
}

fn confirm(question: &str) -> bool {
    eprint!("{question} [y/N] ");
    let _ = std::io::stderr().flush();
    let mut line = String::new();
    std::io::stdin().lock().read_line(&mut line).is_ok()
        && matches!(
            line.trim().to_lowercase().as_str(),
            "y" | "yes" | "e" | "evet"
        )
}

fn cmd_kill(a: &Args) -> Result<ExitCode, AppError> {
    let entries = scan(Detail::Full)?;
    let (pid, start_ms, label) = match (a.pid, a.start_ms, a.port) {
        (Some(pid), Some(start), _) => {
            let name = entries
                .iter()
                .find(|e| e.pid == pid)
                .map(|e| e.process_name.clone())
                .unwrap_or_else(|| "?".into());
            (pid, start, format!("{name} (PID {pid})"))
        }
        (Some(_), None, _) => {
            return Err(AppError::Other(
                "--pid needs --start-ms (from `list --json`)".into(),
            ))
        }
        (None, _, Some(port)) => {
            let mut owners: Vec<&PortEntry> = entries
                .iter()
                .filter(|e| {
                    e.local_port == port && (e.state == "LISTEN" || e.protocol == Protocol::Udp)
                })
                .collect();
            owners.sort_by_key(|e| e.pid);
            owners.dedup_by_key(|e| e.pid);
            match owners.as_slice() {
                [] => return Err(AppError::NotFound),
                [e] => {
                    if let Some(cmd) = &e.cmdline {
                        eprintln!("port {port}: {cmd}");
                    }
                    (
                        e.pid,
                        e.process_start_ms,
                        format!("{} (PID {}) on port {port}", e.process_name, e.pid),
                    )
                }
                many => {
                    eprintln!("port {port} is used by several processes; choose one with --pid PID --start-ms MS:");
                    for e in many {
                        eprintln!(
                            "  --pid {} --start-ms {}   {}",
                            e.pid, e.process_start_ms, e.process_name
                        );
                    }
                    return Ok(ExitCode::from(2));
                }
            }
        }
        _ => {
            return Err(AppError::Other(
                "kill needs a PORT or --pid/--start-ms".into(),
            ))
        }
    };
    if !a.yes {
        if !std::io::stdin().is_terminal() {
            eprintln!("refusing to end {label} without confirmation: not a terminal (pass --yes)");
            return Ok(ExitCode::from(2));
        }
        if !confirm(&format!("End {label}?")) {
            eprintln!("cancelled");
            return Ok(ExitCode::SUCCESS);
        }
    }
    let mut sys = System::new();
    let report = kill::kill(&mut sys, pid, start_ms, a.tree)?;
    println!(
        "ended PID {}",
        report
            .killed
            .iter()
            .map(u32::to_string)
            .collect::<Vec<_>>()
            .join(", ")
    );
    for f in &report.failed {
        eprintln!(
            "could not end child {} (PID {}): {}",
            f.name, f.pid, f.message
        );
    }
    Ok(ExitCode::SUCCESS)
}

fn exit_code(e: &AppError) -> u8 {
    match e {
        AppError::NotFound => 3,
        AppError::AccessDenied => 4,
        AppError::Changed => 5,
        AppError::Protected(_) => 6,
        _ => 1,
    }
}

fn main() -> ExitCode {
    let argv: Vec<String> = std::env::args().skip(1).collect();
    let args = match parse(&argv) {
        Ok(a) => a,
        Err(e) => {
            eprintln!("peron-cli: {e}\n\n{HELP}");
            return ExitCode::from(2);
        }
    };
    let result = match args.command.as_str() {
        "list" | "ls" => cmd_list(&args),
        "export" => cmd_export(&args),
        "watch" => cmd_watch(&args),
        "kill" => cmd_kill(&args),
        "version" => {
            println!("peron-cli {VERSION}");
            Ok(ExitCode::SUCCESS)
        }
        "help" => {
            println!("{HELP}");
            Ok(ExitCode::SUCCESS)
        }
        other => {
            eprintln!("peron-cli: unknown command: {other}\n\n{HELP}");
            return ExitCode::from(2);
        }
    };
    match result {
        Ok(code) => code,
        Err(e) => {
            // English on purpose: scripts and the remote view parse stderr / exit codes.
            let msg = match &e {
                AppError::NotFound => "no such process or port".to_string(),
                AppError::AccessDenied => "access denied (run as root/administrator)".to_string(),
                AppError::Changed => "the process changed (PID reused); list again".to_string(),
                AppError::Protected(n) => format!("{n} is a protected system process"),
                other => other.to_string(),
            };
            eprintln!("peron-cli: {msg}");
            ExitCode::from(exit_code(&e))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn p(s: &str) -> Result<Args, String> {
        parse(&s.split_whitespace().map(String::from).collect::<Vec<_>>())
    }

    #[test]
    fn defaults_to_list() {
        assert_eq!(p("").unwrap().command, "list");
        assert_eq!(p("--json").unwrap().format, Some(Format::Json));
    }

    #[test]
    fn list_flags() {
        let a = p("list --all --udp --system --format csv --lang tr").unwrap();
        assert!(a.all && a.udp && a.system);
        assert_eq!((a.format, a.lang), (Some(Format::Csv), Some(Lang::Tr)));
    }

    #[test]
    fn kill_forms() {
        let a = p("kill 3000 --yes --tree").unwrap();
        assert_eq!((a.port, a.yes, a.tree), (Some(3000), true, true));
        let b = p("kill --pid 42 --start-ms 1700000000000 --yes").unwrap();
        assert_eq!((b.pid, b.start_ms), (Some(42), Some(1_700_000_000_000)));
    }

    #[test]
    fn rejects_bad_input() {
        assert!(p("kill 0").is_err());
        assert!(p("kill 99999").is_err());
        assert!(p("list --format xml").is_err());
        assert!(p("watch --interval 0").is_err());
        assert!(p("list --bogus").is_err());
        assert!(p("kill 1 2").is_err());
    }

    #[test]
    fn exit_codes_match_remote_classifier() {
        use peron_core::remote::{classify, RemoteErrorKind};
        assert_eq!(
            classify(Some(exit_code(&AppError::NotFound) as i32), ""),
            RemoteErrorKind::NotFound
        );
        assert_eq!(
            classify(Some(exit_code(&AppError::AccessDenied) as i32), ""),
            RemoteErrorKind::AccessDenied
        );
        assert_eq!(
            classify(Some(exit_code(&AppError::Changed) as i32), ""),
            RemoteErrorKind::Changed
        );
        assert_eq!(
            classify(Some(exit_code(&AppError::Protected("x".into())) as i32), ""),
            RemoteErrorKind::Protected
        );
    }
}
