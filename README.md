<p align="center"><img src="design/peron-icon.png" width="96" alt="Peron logo"></p>

<h1 align="center">Peron</h1>

<p align="center">
  Localhost port manager for Windows and Linux — see which process holds a port, who started it,
  how long it has been open, and close the forgotten ones.<br>
  <b>Beta</b> · Free · <a href="https://burakaltiparmak.dev/products/peron">burakaltiparmak.dev/products/peron</a>
</p>

---

## Features
- Lists listening TCP ports and bound UDP sockets (or every connection), IPv4 + IPv6 merged per process.
- Owning process, **who started it** (parent chain), **project folder**, command line, and **when the port was opened**.
- CPU, memory and disk I/O per process.
- **Close a port** by ending its process — always behind a confirmation dialog; protected system processes can't be ended; PID-reuse safe.
- Lives in the system tray, **reminds you** about dev servers left open too long.
- Very light in the background: closing the window unloads the UI (≈ 4–6 MB RAM, ~0 % CPU in the tray).
- Turkish and English, light/dark theme, keyboard shortcuts.
- **Export** the visible ports as TXT / CSV / JSON (timestamped file) or copy them as a table.
- **History**: a timeline of ports opening and closing, with an alert when a process opens a port reachable from the network.
- **Servers over SSH**: pick a server in the machine selector and see/close its ports with the same UI — uses your `ssh` client and keys, no agent or open port on the server.
- **peron-cli** for terminals and headless servers:
  ```sh
  peron-cli list                 # listening ports (table)   --all --udp --system --json
  peron-cli export --format csv  # peron-<host>-<UTC time>.csv
  peron-cli watch                # print ports as they open/close
  peron-cli kill 3000            # asks for confirmation (--yes in scripts)
  ```
  Linux: included in the .deb/.rpm (`/usr/bin/peron-cli`) or as a static binary (`peron-cli-x86_64-linux`, runs on any x86_64 distro) on the release page.

## Download
| Platform | Where |
|---|---|
| Windows 10/11 (x64) | [Microsoft Store](https://apps.microsoft.com/detail/9MVV5PRQ9J6D) · [GitHub Releases](https://github.com/mburakaltiparmak/peron/releases) (`Peron_<version>_x64-setup.exe`) |
| Linux x64 | [GitHub Releases](https://github.com/mburakaltiparmak/peron/releases): `.deb`, `.rpm`, `.AppImage` |

The direct-download Windows installer is not code-signed yet, so SmartScreen may show "Windows protected your PC" → *More info* → *Run anyway*. The Microsoft Store version is signed by Microsoft.

## Privacy
No accounts, ads or telemetry. Port and process data never leave your device. See [PRIVACY.md](PRIVACY.md).

## Support
[SUPPORT.md](SUPPORT.md) · in-app *Contact* tab · mburakaltiparmak@gmail.com. Security issues: [SECURITY.md](SECURITY.md).

## Building from source
Requirements: Node 22, Rust (stable, MSVC on Windows), Tauri 2 prerequisites.

```sh
npm ci
npm run tauri dev          # run in development
npm test                   # UI tests (Vitest)
cargo test --manifest-path src-tauri/Cargo.toml
npm run tauri build        # Windows: NSIS installer
npm run build:msix         # Windows: Microsoft Store MSIX (see packaging/msix)
.\docker\linux-build.ps1   # Linux packages via Docker
```

Optional: copy `.env.example` to `.env` and set `WEB3FORMS_FORM_ACCESS_KEY` to enable direct feedback sending.

Manual tests: [docs/MANUAL_TESTS.md](docs/MANUAL_TESTS.md) · Store preparation: [docs/store](docs/store).

## Support
Peron is free. If it saves you time, you can [buy me a coffee ☕](https://buymeacoffee.com/mburakaltiparmak) — entirely optional; it unlocks nothing.

## License
- **Source code and GitHub builds:** [GNU GPL v3.0](LICENSE). You may use, study, change and share
  Peron under its terms; changed versions must stay GPL-3.0 and ship their source.
- **Microsoft Store version:** the same app under the [Store EULA](EULA.md), free of charge.
- **Name and logo:** "Peron" and the Peron logo are trademarks of M. Burak Altıparmak and are not
  covered by the GPL. Publish forks under a different name and without the logo.
- **Contributions** require the [Contributor License Agreement](CLA.md); see [CONTRIBUTING.md](CONTRIBUTING.md).

© 2026 M. Burak Altıparmak

---

## Türkçe
Peron, localhost'ta açık kalan portları; onları hangi sürecin tuttuğunu, kimin başlattığını, ne zamandır açık olduğunu ve ne kadar kaynak tükettiğini gösteren, unutulan portları tek tıkla kapatmanızı sağlayan ücretsiz bir masaüstü uygulamasıdır. İndirme: [Microsoft Store](https://apps.microsoft.com/detail/9MVV5PRQ9J6D), [GitHub Releases](https://github.com/mburakaltiparmak/peron/releases) ve [burakaltiparmak.dev/products/peron](https://burakaltiparmak.dev/products/peron). Gizlilik: [PRIVACY.md](PRIVACY.md) · Destek: [SUPPORT.md](SUPPORT.md) · [Bana bir kahve ısmarla ☕](https://buymeacoffee.com/mburakaltiparmak) (tamamen isteğe bağlı).

**Lisans:** kaynak kod ve GitHub derlemeleri [GPL-3.0](LICENSE); Microsoft Store sürümü [EULA](EULA.md) ile ücretsiz. "Peron" adı ve logosu GPL kapsamında değildir; fork'lar farklı bir adla ve logosuz yayımlanmalıdır. Katkılar [CLA](CLA.md) onayı gerektirir.
