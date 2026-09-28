import type { PortEntry } from "../types";

let seq = 0;

/** A plausible listening TCP socket; override what the test cares about. */
export function port(over: Partial<PortEntry> = {}): PortEntry {
  seq += 1;
  return {
    id: `TCP|127.0.0.1|${3000 + seq}||0|${1000 + seq}`,
    protocol: "TCP",
    ipVersion: 4,
    localAddr: "127.0.0.1",
    localPort: 3000 + seq,
    remoteAddr: null,
    remotePort: null,
    state: "LISTEN",
    pid: 1000 + seq,
    processName: "node.exe",
    exePath: "C:\\Program Files\\nodejs\\node.exe",
    cmdline: "node server.js",
    cwd: "C:\\code\\shop-api",
    project: "shop-api",
    parentPid: 900,
    parentName: "Code.exe",
    openedAtMs: 1_700_000_000_000,
    openedAtSource: "socket",
    processStartMs: 1_700_000_000_000,
    cpuPercent: 0,
    memoryBytes: 50 * 1024 * 1024,
    diskReadBytes: 0,
    diskWrittenBytes: 0,
    isSystem: false,
    isProtected: false,
    isDev: true,
    isLocalhostOnly: true,
    accessible: true,
    ...over,
  };
}
