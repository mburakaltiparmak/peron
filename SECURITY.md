# Security Policy

## Supported versions
Only the latest release (including the latest beta) receives security fixes.

## Reporting a vulnerability
Please **do not open a public issue**. Email **mburakaltiparmak@gmail.com** with the subject
"Peron security" and include:
- affected version and OS,
- steps to reproduce or a proof of concept,
- the impact you expect.

You'll get an acknowledgement within 7 days and a status update within 30 days. Please give us a
reasonable time to release a fix before any public disclosure; we're happy to credit you.

## Scope — things we especially care about
- Any way to end a process **without** the user's explicit confirmation, or to end a protected
  system process.
- Command/URL injection through the Tauri IPC surface (`src-tauri/src/commands.rs`) or the
  opener allow-list (`src-tauri/capabilities/default.json`).
- Content Security Policy bypasses in the WebView.
- Leaks of local data (port lists, command lines, paths) off the device.

## Design notes
- The app runs with normal user rights (`asInvoker`); elevation happens only when the user asks
  (Windows, direct-download build).
- The feedback form's Web3Forms key can only deliver mail to the developer; client-side limits
  are a courtesy, server-side filtering is Web3Forms'.
