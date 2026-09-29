# Release checklist

Legend: ✅ done in the repo · 🧑 needs you (account, hosting, manual test) · ⏳ not yet

## 1. Before every release
- [ ] Version bumped in `package.json`, `src-tauri/Cargo.toml`, `src-tauri/tauri.conf.json`; `CHANGELOG.md` entry
- [ ] `npm test`, `npx tsc --noEmit`, `npm audit --audit-level=high`
- [ ] `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test -- --include-ignored` (Windows + `.\docker\linux-build.ps1 test`)
- [ ] CI green on `main`
- [ ] Tag `v<version>` → CI drafts the GitHub release → review assets → publish (0.1.0-beta.1: published 2026-09-27)
- [ ] Check the portfolio page https://burakaltiparmak.dev/products/peron lists the new release (it reads GitHub Releases)

## 2. Microsoft Store — one-time setup (🧑)
- [x] Partner Center individual developer account (storedeveloper.microsoft.com) — account type can't be changed later
- [x] Reserve the name **Peron**
- [x] Product created as **MSIX or PWA app** (not "EXE or MSI app": that type needs a paid code-signing certificate and a hosted installer URL). Identity in [PARTNER_CENTER.md](PARTNER_CENTER.md)
- [x] Copy *Package/Identity/Name*, *Package/Identity/Publisher*, *PublisherDisplayName* from Product identity into GitHub repo **variables** `MSIX_IDENTITY_NAME`, `MSIX_PUBLISHER`, `MSIX_PUBLISHER_DISPLAY_NAME` (CI fails on `v*` tags until all three are set and non-placeholder)
- [x] Portfolio pages live (2026-09-27): https://burakaltiparmak.dev/products/peron (downloads), `/products/peron/privacy` (privacy), https://burakaltiparmak.dev/contact (support)
- [x] Make the GitHub repo public (release URLs, PRIVACY/SUPPORT mirrors, update check), then protect it:
  - Settings → Rules → Rulesets → new branch ruleset for `main`: *Restrict deletions*, *Block force pushes*,
    *Require a pull request before merging* + *Require review from Code Owners*; add yourself to the bypass list
  - Settings → Actions → General: fork pull request workflows → *Require approval for all external contributors*;
    workflow permissions → *Read repository contents*; *Allow GitHub Actions to create and approve pull requests* off
  - Partner Center → Store listing → *Additional license terms*: link to `EULA.md` (see `LISTING.*.md`)
- [ ] Verify the Web3Forms privacy URL referenced in PRIVACY.md (https://web3forms.com/privacy)
- [x] GitHub secret `WEB3FORMS_FORM_ACCESS_KEY`

## 3. Microsoft Store — every submission
- [x] Download the `peron-msix` artifact (or `npm run build:msix` with the real identity) — 0.1.0-beta.1 submitted and approved 2026-09-29
- [x] Local install test (Developer Mode or `-SelfSign`), see below — real-identity MSIX passed 2026-09-27
- [x] Windows App Certification Kit on the MSIX (latest WACK) — passed 2026-09-27
- [ ] Microsoft Defender scan of the package
- [x] Store listing TR + EN from `docs/store/LISTING.*.md`, screenshots from `docs/store/screenshots/*`, logo from `docs/store/assets`
- [x] Screenshots retaken for 0.1.0-beta.1 (2026-09-26): Store UI (no "Admin" button, no update option), TR + EN, 1920×1080, demo services in `C:\PeronDemo`. Retake when the UI changes visibly.
- [x] Age rating questionnaire (IARC): no user interaction/sharing, no purchases → "3+/Everyone"
- [x] Notes for certification from `docs/store/CERTIFICATION_NOTES.md`
- [x] Declare the use of a secure third-party purchase API (voluntary tips via Buy Me a Coffee, opened in the browser; nothing is unlocked — policy 10.8.2)
- [x] Accessibility claim backed by Narrator + high-contrast tests (2026-09-26)
- [x] First release approved and publicly available in Microsoft Store (2026-09-29): https://apps.microsoft.com/detail/9MVV5PRQ9J6D

## 4. Manual test matrix (Windows 10 x64 + Windows 11 x64)
| Test | NSIS (site/GitHub) | MSIX (Store) |
|---|---|---|
| Silent install `/S` + start | ✅ scripted (`packaging/test-nsis-silent.ps1`) | n/a |
| Silent uninstall, no registry/Run leftovers | ✅ scripted | Windows removes everything |
| Installer language selector (TR/EN) → app language | 🧑 | n/a (Windows language) |
| Standard (non-admin) user install | 🧑 | 🧑 |
| Offline start (no internet) | 🧑 | 🧑 |
| Port list, close a test server, protected process disabled | ✅ automated (Rust live test) + 🧑 UI | 🧑 |
| Tray: close window → tray, reopen, Quit | 🧑 | 🧑 |
| Reminder notification with Peron name/icon, click opens window | ✅ (NSIS, earlier) | 🧑 (package AUMID) |
| Start with Windows on/off | 🧑 (Run key) | 🧑 (StartupTask) |
| Admin relaunch (UAC) | 🧑 | hidden by design |
| Update notice (needs a newer GitHub release) | 🧑 | never shown |
| Web3Forms unreachable → app keeps working, error shown | 🧑 | 🧑 |

### Local MSIX install
```powershell
# Option A: Developer Mode on (Settings → System → For developers)
Add-AppxPackage -Register src-tauri\target\msix\stage\AppxManifest.xml
# Option B: test certificate (admin once)
.\packaging\msix\build-msix.ps1 -SelfSign
# import the printed certificate into Local Machine → Trusted People, then double-click the .msix
# remove afterwards:
Get-AppxPackage *Peron* | Remove-AppxPackage
```
