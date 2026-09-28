# Code signing without a budget

## Decision (2026-09-25)
No budget for a code-signing certificate → **Microsoft Store via MSIX**.

| Channel | Package | Signed by | Cost |
|---|---|---|---|
| Microsoft Store | MSIX (`npm run build:msix`) | **Microsoft**, on submission | Free (individual developer account has no fee) |
| Site / GitHub | NSIS `.exe`, Linux `.deb/.rpm/.AppImage` | — (unsigned) | Free |

Why not the EXE/MSI Store route? Microsoft requires the installer *and* every PE file inside it to be signed with a certificate chaining to the Microsoft Trusted Root Program — i.e. a paid OV/EV certificate or Azure Trusted Signing (paid, identity-verified). MSIX packages submitted to the Store are re-signed by Microsoft, so we need no certificate of our own.

## Consequences
- **Unsigned direct downloads** trigger SmartScreen ("Windows protected your PC" → More info → Run anyway) until the file builds reputation. Documented in README/SUPPORT. The Store build has no warning.
- **Local MSIX testing** needs either Windows Developer Mode (`Add-AppxPackage -Register <stage>\AppxManifest.xml`) or a self-signed test certificate (`build-msix.ps1 -SelfSign`, then import the certificate into *Local Machine → Trusted People*). Neither is needed for the Store submission.
- Linux packages are normally unsigned (AppImage/deb from GitHub); checksums are published with each release.

## If a budget appears later
- Azure Trusted Signing: add a `bundle.windows.signCommand` using `trusted-signing-cli` and CI secrets (`AZURE_CLIENT_ID`, `AZURE_TENANT_ID`, `AZURE_CLIENT_SECRET`, endpoint/account/profile).
- Classic OV certificate (e.g. Certum, Sectigo): set `bundle.windows.certificateThumbprint`, `digestAlgorithm: "sha256"`, `timestampUrl` in a CI-only Tauri config; import the PFX from a secret.
Either removes the SmartScreen warning for direct downloads over time.
