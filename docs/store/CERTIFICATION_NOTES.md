# Notes for certification (Partner Center → Submission options → Notes for certification)

Paste the block below (≤ 2,000 characters).

---

Peron is a developer tool: it lists local TCP/UDP ports and the processes that own them (Windows IP Helper API, read-only) and can end a process to free a port. No generative AI features.

Process termination is always user-confirmed:
- Only after "Close" + a dialog showing name, PID, command line and affected ports. Nothing is ended automatically.
- Protected system processes (System, csrss, lsass, services, svchost, …) can't be ended; the button is disabled.
- PID and start time are re-checked before ending, so a reused PID is never hit.
- The Store build runs with normal user rights and never requests elevation.

How to test:
1. In a terminal run: python -m http.server 8765
2. Start Peron: port 8765 appears with owner, parent, project folder and "open for" time.
3. Click "Close" on it, confirm "End process": the port disappears.
4. Untick "Hide system processes": svchost.exe rows have a disabled "Close".
5. Close the window: Peron stays in the tray; the tray icon reopens it, "Quit" exits.
6. Settings → Reminders → "When open longer than" = 1: a notification appears in ~2 min.
7. Settings → Startup → "Start with Windows" uses the package StartupTask.
8. History tab lists opened/closed ports; Export saves TXT/CSV/JSON to Downloads.

Network use (all user-initiated, no telemetry, works offline):
- Contact form: sends the user's message via Web3Forms (HTTPS).
- Optional remote view: runs the user's own ssh client against servers the user adds, with the user's keys; no Peron server involved.
- "Buy me a coffee" opens buymeacoffee.com in the browser: a voluntary tip that unlocks nothing (declared as third-party purchase, policy 10.8.2).

Privacy policy: https://burakaltiparmak.dev/products/peron/privacy
