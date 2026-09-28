# Changelog

All notable changes to Peron. Versions follow [SemVer](https://semver.org/); pre-releases are
tagged `vX.Y.Z-beta.N`. The Microsoft Store package uses the mapped MSIX version shown in
parentheses (see `packaging/msix/build-msix.ps1`).

## [0.1.0-beta.1] — 2026-09-27 (MSIX 0.1.1.0)

First public beta.

### Added
- Port list: listening TCP / bound UDP or all connections; IPv4 + IPv6 grouped per process.
- Process details: owner, parent chain ("started by"), project folder, command line, working
  directory, socket open time (Windows), CPU / RAM / disk I/O.
- Close a port by ending its process, behind a confirmation dialog; protected system processes
  and PID reuse are guarded; optional process-tree end.
- System tray with open-port count and quick access to the longest-open dev ports.
- Reminders for dev ports left open longer than a configurable threshold.
- Adaptive background scanning; the window's WebView is unloaded in the tray.
- Turkish / English UI (language chosen in the Windows installer, changeable in Settings), themes,
  keyboard shortcuts and navigation.
- Contact tab with feedback form (Web3Forms), privacy policy and support links.
- "New version available" notice for direct downloads (not in the Store build).
- Windows (NSIS, Microsoft Store MSIX) and Linux (deb, rpm, AppImage) packages.
- Export visible ports as TXT / CSV / JSON (timestamped file) or copy as a table.
- History tab: port open/close timeline (local, 30 days) with export; alert when a new port is
  reachable from the network.
- Remote view over SSH (machine selector, Servers dialog): list and close ports on servers.
- `peron-cli`: `list`, `export`, `watch`, `kill` — static Linux binary, also in the .deb/.rpm.
- Theme changes preview live and apply to the window title bar; screen reader names and high
  contrast support for selections.
- Diagnostics log in `Documents\Peron\Logs` on Windows ("Open log folder" in Settings).
- Optional "Buy me a coffee" link in the Contact tab (unlocks nothing).

### License
- Source code and GitHub builds: GPL-3.0. Microsoft Store version: [EULA](EULA.md).
  Contributions need the [CLA](CLA.md).
