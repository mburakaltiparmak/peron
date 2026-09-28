//! System / dev process classification. The protected lists must never shrink: they are what
//! keeps Peron from ending processes the OS needs.

#[cfg(windows)]
const PROTECTED: &[&str] = &[
    "system",
    "system idle process",
    "idle",
    "registry",
    "secure system",
    "memory compression",
    "smss.exe",
    "csrss.exe",
    "wininit.exe",
    "winlogon.exe",
    "services.exe",
    "lsass.exe",
    "lsaiso.exe",
    "svchost.exe",
    "dwm.exe",
    "fontdrvhost.exe",
];

/// Well-known system components that live outside %SystemRoot%.
#[cfg(windows)]
const KNOWN_SYSTEM: &[&str] = &[
    "msmpeng.exe",
    "nissrv.exe",
    "mpdefendercoreservice.exe",
    "spoolsv.exe",
    "searchindexer.exe",
    "wmiprvse.exe",
];

/// Linux: init, session/display stack and core daemons whose loss breaks the desktop or login.
#[cfg(target_os = "linux")]
const PROTECTED: &[&str] = &[
    "systemd",
    "init",
    "kthreadd",
    "dbus-daemon",
    "dbus-broker",
    "polkitd",
    "xorg",
    "xwayland",
    "gnome-shell",
    "gnome-session-binary",
    "kwin_wayland",
    "kwin_x11",
    "plasmashell",
    "gdm",
    "gdm3",
    "sddm",
    "lightdm",
    "networkmanager",
    "wpa_supplicant",
    "sshd",
    "pipewire",
    "wireplumber",
    "pulseaudio",
];

/// Linux daemons that are part of the OS but safe to list as "system" (hidden by default).
#[cfg(target_os = "linux")]
const KNOWN_SYSTEM: &[&str] = &[
    "avahi-daemon",
    "cupsd",
    "cups-browsed",
    "chronyd",
    "rsyslogd",
    "dnsmasq",
    "rpcbind",
    "containerd",
    "snapd",
    "packagekitd",
    "colord",
    "kdeconnectd",
];

const DEV: &[&str] = &[
    "docker-proxy",
    "php-fpm",
    "puma",
    "node",
    "deno",
    "bun",
    "python",
    "pythonw",
    "python3",
    "py",
    "uvicorn",
    "gunicorn",
    "java",
    "javaw",
    "dotnet",
    "iisexpress",
    "ruby",
    "php",
    "php-cgi",
    "go",
    "air",
    "cargo",
    "esbuild",
    "docker",
    "dockerd",
    "com.docker.backend",
    "wslrelay",
    "postgres",
    "mysqld",
    "mariadbd",
    "redis-server",
    "mongod",
    "memurai",
    "nginx",
    "httpd",
    "caddy",
    "beam.smp",
    "erl",
    "ngrok",
    "hugo",
    "jupyter",
    "ollama",
];

fn lower(name: &str) -> String {
    name.to_ascii_lowercase()
}

fn stem(name: &str) -> String {
    let n = lower(name);
    n.strip_suffix(".exe").map(str::to_owned).unwrap_or(n)
}

/// PID 0 is also "unknown owner" (Linux without root), which must never be killable either.
pub fn is_protected(pid: u32, name: &str) -> bool {
    let n = lower(name);
    #[cfg(target_os = "linux")]
    if n.starts_with("systemd") {
        return true;
    }
    pid <= 4 || PROTECTED.contains(&n.as_str())
}

pub fn is_system(pid: u32, name: &str, exe: Option<&str>) -> bool {
    if is_protected(pid, name) || KNOWN_SYSTEM.contains(&lower(name).as_str()) {
        return true;
    }
    exe.is_some_and(|e| is_system_path(&lower(e)))
}

#[cfg(windows)]
fn is_system_path(exe: &str) -> bool {
    let root = std::env::var("SystemRoot").unwrap_or_else(|_| r"C:\Windows".into());
    exe.starts_with(&format!("{}\\", lower(&root)))
}

#[cfg(target_os = "linux")]
fn is_system_path(exe: &str) -> bool {
    [
        "/usr/sbin/",
        "/sbin/",
        "/usr/lib/systemd/",
        "/lib/systemd/",
        "/usr/libexec/",
    ]
    .iter()
    .any(|p| exe.starts_with(p))
}

pub fn is_dev(name: &str) -> bool {
    DEV.contains(&stem(name).as_str())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(target_os = "linux")]
    #[test]
    fn linux() {
        assert!(is_protected(1, "systemd"));
        assert!(is_protected(900, "systemd-resolved"));
        assert!(is_protected(900, "gnome-shell"));
        assert!(!is_protected(900, "node"));
        assert!(is_system(900, "cupsd", None));
        assert!(is_system(900, "foo", Some("/usr/sbin/foo")));
        assert!(!is_system(
            900,
            "node",
            Some("/home/u/.nvm/versions/node/v22/bin/node")
        ));
        assert!(is_dev("node") && is_dev("python3") && is_dev("docker-proxy"));
    }

    #[cfg(windows)]
    #[test]
    fn protected() {
        assert!(is_protected(0, "System Idle Process"));
        assert!(is_protected(4, "System"));
        assert!(is_protected(1234, "svchost.exe"));
        assert!(is_protected(1234, "LSASS.EXE"));
        assert!(!is_protected(1234, "node.exe"));
    }

    #[cfg(windows)]
    #[test]
    fn system() {
        assert!(is_system(900, "spoolsv.exe", None));
        assert!(is_system(
            900,
            "foo.exe",
            Some(r"C:\Windows\System32\foo.exe")
        ));
        assert!(!is_system(
            900,
            "node.exe",
            Some(r"C:\Program Files\nodejs\node.exe")
        ));
        assert!(!is_system(
            900,
            "windowsapp.exe",
            Some(r"C:\WindowsApps\x.exe")
        ));
    }

    #[test]
    fn dev() {
        assert!(is_dev("node.exe"));
        assert!(is_dev("Python.EXE"));
        assert!(is_dev("com.docker.backend.exe"));
        assert!(!is_dev("chrome.exe"));
    }
}
