# Partner Center — Peron (values to enter)

Everything entered in Partner Center for the Microsoft Store submission, in page order. Long texts
live in their own files and are linked. Product type: **MSIX or PWA app** — never "EXE or MSI app"
(that type needs a paid code-signing certificate and a self-hosted installer URL).

## Product identity (Product management → Product identity)

| Field | Value |
|---|---|
| Package/Identity/Name | `M.BurakAltiparmak.Peron` |
| Package/Identity/Publisher | `CN=9B72CAB6-7945-401D-A620-65C1A1B29120` |
| Package/Properties/PublisherDisplayName | `M. Burak Altiparmak` |
| Package Family Name (PFN) | `M.BurakAltiparmak.Peron_jtmghbyjncjp8` |
| Store ID | `9MVV5PRQ9J6D` → https://apps.microsoft.com/detail/9MVV5PRQ9J6D (after going live) |

The first three are the defaults of `packaging/msix/build-msix.ps1` and the GitHub repository variables
`MSIX_IDENTITY_NAME`, `MSIX_PUBLISHER`, `MSIX_PUBLISHER_DISPLAY_NAME`. They must match exactly.

## Pricing and availability
- Price: **Free**. Markets: all. Visibility: public.

## Properties
| Field | Value |
|---|---|
| Category / Subcategory | Developer tools / Utilities (Networking if Utilities is missing) |
| Accesses, collects or transmits personal information | **Yes** (optional name/email in the feedback form) |
| Privacy policy URL | https://burakaltiparmak.dev/products/peron/privacy (must be live before submitting) |
| Website | https://burakaltiparmak.dev/products/peron |
| Support contact | https://burakaltiparmak.dev/products/peron/support (or mburakaltiparmak@gmail.com) |
| Generative AI features | **No** (unchecked) |

Product declarations:

| Declaration | Choice | Why |
|---|---|---|
| Allows purchases without the Microsoft Store commerce system | ✅ | Buy Me a Coffee tip (policy 10.8.2) |
| Tested to meet accessibility guidelines | ✅ | Narrator, high contrast, keyboard tested |
| Can be installed to alternate drives or removable storage | ✅ | — |
| Windows can include data in automatic backups to OneDrive | ⬜ | Privacy policy: history stays on the device |
| Depends on non-Microsoft drivers or NT services | ⬜ | — |
| Windows 10 in S mode | ⬜ | Not tested |

Display mode: **leave every box unchecked** (Peron is a 2D desktop app). Any Mixed Reality option here makes
Partner Center demand a "Windows Mixed Reality immersive headset" hardware choice below.

System requirements: keyboard ✅, mouse ✅ (minimum and recommended); everything else not specified.
Architecture (x64) and minimum Windows (10.0.17763) come from the package.

## Age ratings (IARC)
- Type: **Utility, Productivity, Communication, or Other**. Contact email: mburakaltiparmak@gmail.com
- All content questions (violence, sexuality, language, drugs, gambling…): **No**
- Users interact with other users: **No** (feedback goes only to the developer)
- Shares physical location: **No** · Purchases of digital goods: **No** · Ads: **No**
- Unrestricted internet access: **No** · Shares personal info with third parties / social: **No**
- Expected result: **3+** (PEGI 3, ESRB Everyone, USK 0)

## Packages
- Upload `src-tauri/target/msix/store/Peron_<version>_x64.msix` (**unsigned**; the Store signs it), built with
  `.\packaging\msix\build-msix.ps1` — or the `peron-msix` artifact of a `v*` tag build.
- Device families: **Windows 10/11 Desktop** only (Xbox, Team, Holographic unchecked).
- Every submission needs a higher version (0.1.0-beta.1 → 0.1.1.0, beta.2 → 0.1.2.0, 0.1.0 → 0.1.999.0).

## Store listings (Turkish + English)
Texts, search terms, captions and screenshot order: [LISTING.tr.md](LISTING.tr.md) · [LISTING.en.md](LISTING.en.md).
Screenshots: `screenshots/tr`, `screenshots/en`. Logo: `assets/`.
Additional license terms: text of [EULA.md](../../EULA.md) (or its GitHub link once the repository is public).

## Submission options
- Notes for certification: the block in [CERTIFICATION_NOTES.md](CERTIFICATION_NOTES.md).
- Restricted capability justification (`runFullTrust`; the package upload shows a "requires approval" warning,
  which is expected for Win32 apps). The field takes **at most 500 characters**; this text is 490, one line:

  ```
  Peron is a Win32 desktop app (Tauri) packaged as MSIX. It needs full trust for Win32 APIs unavailable in AppContainer: the IP Helper API (GetExtendedTcpTable/UdpTable) to list open ports and their owning PIDs; process details (path, command line, parent, CPU/RAM); and TerminateProcess to free a port, only after the user confirms in a dialog. System processes are protected. Also tray, StartupTask and notifications. Runs with normal user rights, never elevated; no data leaves the device.
  ```
- Third-party purchase API: declared (voluntary tips via Buy Me a Coffee, opened in the browser; unlocks nothing).
