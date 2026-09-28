//! Remote view over SSH: runs `peron-cli` on another machine with the system `ssh` client and
//! reads its JSON. No agent, daemon or open port on the server; authentication is the user's
//! own SSH setup (keys / agent). Nothing passes through any Peron server.
//!
//! Security rules (keep them):
//! - The target is validated (`is_valid_target`) and placed after `--`, so it can never be parsed
//!   as an ssh option (e.g. `-oProxyCommand=…`).
//! - The remote command consists only of fixed words and numbers — ssh joins it into a shell
//!   string on the server, so no user text may ever be added to it.
//! - `BatchMode=yes`: never prompts (no password dialogs); unknown host keys are *not*
//!   auto-accepted — the user connects once in a terminal to verify the fingerprint.

use std::process::{Command, Stdio};

use serde::{Deserialize, Serialize};

use crate::export::{Snapshot, SCHEMA_VERSION};

pub const CLI: &str = "peron-cli";

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RemoteHost {
    pub id: String,
    /// Display name, e.g. "web-01 (prod)".
    pub name: String,
    /// `user@host`, `host` or an alias from ~/.ssh/config.
    pub target: String,
    #[serde(default)]
    pub port: Option<u16>,
}

impl RemoteHost {
    pub fn is_valid(&self) -> bool {
        is_valid_target(&self.target)
            && !self.name.trim().is_empty()
            && self.name.len() <= 80
            && self.port != Some(0)
    }
}

/// `user@host.example`, `host`, `10.0.0.5`, `deploy@web-01`, ssh-config aliases.
pub fn is_valid_target(t: &str) -> bool {
    !t.is_empty()
        && t.len() <= 255
        && !t.starts_with('-')
        && t.chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '-' | '@' | ':'))
        && t.matches('@').count() <= 1
}

/// Full argument list for `ssh` (without the program name).
pub fn ssh_args(host: &RemoteHost, remote: &[String]) -> Vec<String> {
    let mut args: Vec<String> = [
        "-o",
        "BatchMode=yes",
        "-o",
        "ConnectTimeout=8",
        "-o",
        "ServerAliveInterval=5",
        "-o",
        "ServerAliveCountMax=2",
    ]
    .into_iter()
    .map(String::from)
    .collect();
    if let Some(p) = host.port {
        args.push("-p".into());
        args.push(p.to_string());
    }
    args.push("--".into());
    args.push(host.target.clone());
    args.push(CLI.into());
    args.extend(remote.iter().cloned());
    args
}

/// `peron-cli list` arguments: everything, unfiltered — the GUI filters like for local data.
pub fn list_args() -> Vec<String> {
    ["list", "--format", "json", "--all", "--udp", "--system"]
        .into_iter()
        .map(String::from)
        .collect()
}

pub fn kill_args(pid: u32, start_ms: i64, tree: bool) -> Vec<String> {
    let mut a: Vec<String> = vec![
        "kill".into(),
        "--pid".into(),
        pid.to_string(),
        "--start-ms".into(),
        start_ms.to_string(),
        "--yes".into(),
    ];
    if tree {
        a.push("--tree".into());
    }
    a
}

/// What went wrong, for a localized message in the UI.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum RemoteErrorKind {
    /// No `ssh` client on this computer.
    SshMissing,
    InvalidHost,
    Auth,
    /// Unknown/changed host key: connect once in a terminal.
    HostKey,
    Unreachable,
    /// `peron-cli` not installed / not in PATH on the server.
    CliMissing,
    BadOutput,
    /// The server's peron-cli prints a different snapshot schema than this app reads.
    IncompatibleVersion,
    // Mapped from peron-cli exit codes on `kill`:
    AccessDenied,
    NotFound,
    Changed,
    Protected,
    Other,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RemoteError {
    pub kind: RemoteErrorKind,
    /// Last line of ssh/peron-cli stderr, for the details line.
    pub detail: String,
}

/// peron-cli exit codes (see crates/cli): 3 not found, 4 access denied, 5 changed, 6 protected.
pub fn classify(exit: Option<i32>, stderr: &str) -> RemoteErrorKind {
    let s = stderr.to_ascii_lowercase();
    match exit {
        Some(3) => RemoteErrorKind::NotFound,
        Some(4) => RemoteErrorKind::AccessDenied,
        Some(5) => RemoteErrorKind::Changed,
        Some(6) => RemoteErrorKind::Protected,
        Some(127) => RemoteErrorKind::CliMissing,
        _ if s.contains("host key verification failed")
            || s.contains("remote host identification has changed") =>
        {
            RemoteErrorKind::HostKey
        }
        _ if s.contains("permission denied") || s.contains("too many authentication failures") => {
            RemoteErrorKind::Auth
        }
        _ if s.contains("could not resolve")
            || s.contains("connection refused")
            || s.contains("timed out")
            || s.contains("no route to host")
            || s.contains("network is unreachable")
            || s.contains("connection closed") =>
        {
            RemoteErrorKind::Unreachable
        }
        _ if s.contains("peron-cli: command not found")
            || s.contains("peron-cli: not found")
            || s.contains("is not recognized") =>
        {
            RemoteErrorKind::CliMissing
        }
        _ => RemoteErrorKind::Other,
    }
}

fn run(host: &RemoteHost, remote: &[String]) -> Result<String, RemoteError> {
    if !host.is_valid() {
        return Err(RemoteError {
            kind: RemoteErrorKind::InvalidHost,
            detail: host.target.clone(),
        });
    }
    let mut cmd = Command::new("ssh");
    cmd.args(ssh_args(host, remote))
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(0x0800_0000); // CREATE_NO_WINDOW: no console flash
    }
    let out = cmd.output().map_err(|e| RemoteError {
        kind: if e.kind() == std::io::ErrorKind::NotFound {
            RemoteErrorKind::SshMissing
        } else {
            RemoteErrorKind::Other
        },
        detail: e.to_string(),
    })?;
    let stderr = String::from_utf8_lossy(&out.stderr);
    if !out.status.success() {
        let detail = stderr
            .lines()
            .rev()
            .find(|l| !l.trim().is_empty())
            .unwrap_or("")
            .trim()
            .to_string();
        return Err(RemoteError {
            kind: classify(out.status.code(), &stderr),
            detail,
        });
    }
    Ok(String::from_utf8_lossy(&out.stdout).into_owned())
}

/// Reads `peron-cli list --format json`. The envelope is checked before the entries so a CLI with a
/// different schema reports `IncompatibleVersion` (update one side) instead of a vague parse error.
pub fn parse_snapshot(json: &str) -> Result<Snapshot, RemoteError> {
    let bad = |detail: String| RemoteError {
        kind: RemoteErrorKind::BadOutput,
        detail,
    };
    let value: serde_json::Value =
        serde_json::from_str(json.trim()).map_err(|e| bad(e.to_string()))?;
    if value.get("tool").and_then(|t| t.as_str()) != Some(CLI) {
        return Err(bad("not peron-cli output".into()));
    }
    let schema = value.get("schemaVersion").and_then(|v| v.as_u64());
    if schema != Some(u64::from(SCHEMA_VERSION)) {
        let version = value.get("version").and_then(|v| v.as_str()).unwrap_or("?");
        return Err(RemoteError {
            kind: RemoteErrorKind::IncompatibleVersion,
            detail: format!(
                "peron-cli {version}: schemaVersion {} (expected {SCHEMA_VERSION})",
                schema.map_or("?".to_string(), |s| s.to_string())
            ),
        });
    }
    serde_json::from_value(value).map_err(|e| bad(e.to_string()))
}

/// Lists the server's sockets (blocking; ~connect time + 0.3 s).
pub fn fetch(host: &RemoteHost) -> Result<Snapshot, RemoteError> {
    parse_snapshot(&run(host, &list_args())?)
}

/// Ends a process on the server. PID + start time guard against PID reuse, exactly like locally;
/// the user already confirmed in the GUI dialog, hence `--yes`.
pub fn kill(host: &RemoteHost, pid: u32, start_ms: i64, tree: bool) -> Result<(), RemoteError> {
    run(host, &kill_args(pid, start_ms, tree)).map(|_| ())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn host(target: &str) -> RemoteHost {
        RemoteHost {
            id: "h1".into(),
            name: "web".into(),
            target: target.into(),
            port: Some(2222),
        }
    }

    #[test]
    fn targets() {
        for ok in [
            "web-01",
            "deploy@web-01.example.com",
            "root@10.0.0.5",
            "my_alias",
            "u@fe80::1",
        ] {
            assert!(is_valid_target(ok), "{ok}");
        }
        for bad in [
            "",
            "-oProxyCommand=calc",
            "a b",
            "u@h;rm -rf /",
            "u@h$(x)",
            "a@b@c",
            "h\n",
            "`x`",
        ] {
            assert!(!is_valid_target(bad), "{bad:?}");
        }
    }

    #[test]
    fn args_put_target_after_double_dash() {
        let a = ssh_args(&host("deploy@web-01"), &list_args());
        let dd = a.iter().position(|x| x == "--").unwrap();
        assert_eq!(a[dd + 1], "deploy@web-01");
        assert_eq!(a[dd + 2], "peron-cli");
        assert!(a.contains(&"BatchMode=yes".to_string()));
        assert_eq!(&a[a.iter().position(|x| x == "-p").unwrap() + 1], "2222");
        assert!(!a.iter().any(|x| x.contains("StrictHostKeyChecking=no")));
    }

    #[test]
    fn kill_args_are_numbers_only() {
        assert_eq!(
            kill_args(42, 1700, true).join(" "),
            "kill --pid 42 --start-ms 1700 --yes --tree"
        );
    }

    #[test]
    fn errors() {
        assert_eq!(
            classify(Some(255), "root@x: Permission denied (publickey)."),
            RemoteErrorKind::Auth
        );
        assert_eq!(
            classify(Some(255), "Host key verification failed."),
            RemoteErrorKind::HostKey
        );
        assert_eq!(
            classify(Some(255), "ssh: Could not resolve hostname nope"),
            RemoteErrorKind::Unreachable
        );
        assert_eq!(
            classify(Some(127), "bash: line 1: peron-cli: command not found"),
            RemoteErrorKind::CliMissing
        );
        assert_eq!(classify(Some(4), ""), RemoteErrorKind::AccessDenied);
        assert_eq!(classify(Some(1), "boom"), RemoteErrorKind::Other);
    }

    /// Parses JSON captured from a real `peron-cli` over SSH by docker/remote-e2e.ps1 — proves the
    /// GUI's parser reads what the (Linux, static) CLI prints.
    /// Runs only with PERON_REMOTE_JSON set (e.g. `$env:PERON_REMOTE_JSON="$env:TEMP\peron-remote-e2e\remote.json"`).
    #[test]
    fn parses_real_remote_output() {
        let Ok(path) = std::env::var("PERON_REMOTE_JSON") else {
            return; // not in an e2e run
        };
        let text = std::fs::read_to_string(&path).expect("run docker/remote-e2e.ps1 first");
        let snap = parse_snapshot(text.trim_start_matches('\u{feff}')).expect("valid snapshot");
        assert_eq!(snap.schema_version, 1);
        assert_eq!(snap.tool, "peron-cli");
        assert!(snap
            .entries
            .iter()
            .any(|e| e.local_port == 8765 && e.project.as_deref() == Some("shop-api")));
    }

    #[test]
    fn invalid_host_never_spawns_ssh() {
        let err = fetch(&host("-oProxyCommand=calc")).unwrap_err();
        assert_eq!(err.kind, RemoteErrorKind::InvalidHost);
    }

    fn snapshot_json(edit: impl FnOnce(&mut serde_json::Value)) -> String {
        let mut v = serde_json::to_value(crate::export::tests::snap()).unwrap();
        edit(&mut v);
        v.to_string()
    }

    #[test]
    fn snapshot_schema_is_checked() {
        let ok = parse_snapshot(&snapshot_json(|_| {})).expect("current schema");
        assert_eq!(ok.entries.len(), 2);

        for schema in [
            serde_json::json!(0),
            serde_json::json!(2),
            serde_json::json!("1"),
        ] {
            let err = parse_snapshot(&snapshot_json(|v| v["schemaVersion"] = schema.clone()))
                .unwrap_err();
            assert_eq!(err.kind, RemoteErrorKind::IncompatibleVersion, "{schema}");
            assert!(err.detail.contains("0.1.0-beta.1"), "{}", err.detail);
        }
        let missing = snapshot_json(|v| {
            v.as_object_mut().unwrap().remove("schemaVersion");
        });
        assert_eq!(
            parse_snapshot(&missing).unwrap_err().kind,
            RemoteErrorKind::IncompatibleVersion
        );

        // The desktop app's own export, or some other JSON, isn't a peron-cli reply.
        let other_tool = snapshot_json(|v| v["tool"] = "peron".into());
        assert_eq!(
            parse_snapshot(&other_tool).unwrap_err().kind,
            RemoteErrorKind::BadOutput
        );
        assert_eq!(
            parse_snapshot("[]").unwrap_err().kind,
            RemoteErrorKind::BadOutput
        );
        assert_eq!(
            parse_snapshot("bash: motd").unwrap_err().kind,
            RemoteErrorKind::BadOutput
        );

        // Right envelope, broken entries.
        let broken = snapshot_json(|v| v["entries"] = serde_json::json!([{ "localPort": "x" }]));
        assert_eq!(
            parse_snapshot(&broken).unwrap_err().kind,
            RemoteErrorKind::BadOutput
        );
    }
}
