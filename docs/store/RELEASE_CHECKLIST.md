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
- [ ] Partner Center individual developer account (storedeveloper.microsoft.com) — account type can't be changed later
- [ ] Reserve the name **Peron**
- [x] Product created as **MSIX or PWA app** (not "EXE or MSI app": that type needs a paid code-signing certificate and a hosted installer URL). Identity in [PARTNER_CENTER.md](PARTNER_CENTER.md)
- [x] Copy *Package/Identity/Name*, *Package/Identity/Publisher*, *PublisherDisplayName* from Product identity into GitHub repo **variables** `MSIX_IDENTITY_NAME`, `MSIX_PUBLISHER`, `MSIX_PUBLISHER_DISPLAY_NAME` (CI fails on `v*` tags until all three are set and non-placeholder)
- [x] Portfolio pages live (2026-09-27): https://burakaltiparmak.dev/products/peron (downloads), `/products/peron/privacy` (privacy), https://burakaltiparmak.dev/contact (support)
- [ ] Make the GitHub repo public (release URLs, PRIVACY/SUPPORT mirrors, update check), then protect it:
  - Settings → Rules → Rulesets → new branch ruleset for `main`: *Restrict deletions*, *Block force pushes*,
    *Require a pull request before merging* + *Require review from Code Owners*; add yourself to the bypass list
  - Settings → Actions → General: fork pull request workflows → *Require approval for all external contributors*;
    workflow permissions → *Read repository contents*; *Allow GitHub Actions to create and approve pull requests* off
  - Partner Center → Store listing → *Additional license terms*: link to `EULA.md` (see `LISTING.*.md`)
- [ ] Verify the Web3Forms privacy URL referenced in PRIVACY.md (https://web3forms.com/privacy)
- [ ] GitHub secret `WEB3FORMS_FORM_ACCESS_KEY`

## 3. Microsoft Store — every submission
- [ ] Download the `peron-msix` artifact (or `npm run build:msix` with the real identity) — version must be higher than the last submission
- [ ] Local install test (Developer Mode or `-SelfSign`), see below
- [ ] Windows App Certification Kit on the MSIX (latest WACK)
- [ ] Microsoft Defender scan of the package
- [ ] Store listing TR + EN from `docs/store/LISTING.*.md`, screenshots from `docs/store/screenshots/*`, logo from `docs/store/assets`
- [x] Screenshots retaken for 0.1.0-beta.1 (2026-09-26): Store UI (no "Admin" button, no update option), TR + EN, 1920×1080, demo services in `C:\PeronDemo`. Retake when the UI changes visibly.
- [ ] Age rating questionnaire (IARC): no user interaction/sharing, no purchases → expected "3+/Everyone"
- [ ] Notes for certification from `docs/store/CERTIFICATION_NOTES.md`
- [ ] Declare the use of a secure third-party purchase API (voluntary tips via Buy Me a Coffee, opened in the browser; nothing is unlocked — policy 10.8.2)
- [ ] Accessibility: do **not** claim accessibility support until tested with Narrator + high contrast
- [ ] First release to a **private audience**, test on a real Windows 10 and Windows 11 PC, then public

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
