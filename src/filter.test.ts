import { describe, expect, it } from "vitest";
import { filterPorts, groupRows, matches } from "./filter";
import { port } from "./test/fixtures";

const listen = { viewMode: "listen" as const, showUdp: false, hideSystem: true };

describe("matches", () => {
  it("finds by port, process, project and requires every term", () => {
    const e = port({ localPort: 5173, processName: "node.exe", cwd: "C:\\code\\shop-api" });
    expect(matches(e, "5173")).toBe(true);
    expect(matches(e, "NODE")).toBe(true);
    expect(matches(e, "shop-api")).toBe(true);
    expect(matches(e, "node shop")).toBe(true);
    expect(matches(e, "node python")).toBe(false);
  });
});

describe("groupRows", () => {
  it("merges IPv4 + IPv6 sockets of one process/port and keeps the earliest open time", () => {
    const v4 = port({ pid: 7, localPort: 3000, localAddr: "0.0.0.0", ipVersion: 4, isLocalhostOnly: false, openedAtMs: 200 });
    const v6 = port({ pid: 7, localPort: 3000, localAddr: "[::]", ipVersion: 6, isLocalhostOnly: false, openedAtMs: 100 });
    const rows = groupRows([v4, v6], true);
    expect(rows).toHaveLength(1);
    expect(rows[0].addrs).toEqual(["0.0.0.0", "[::]"]);
    expect(rows[0].ipVersions).toEqual([4, 6]);
    expect(rows[0].openedAtMs).toBe(100);
  });

  it("keeps rows separate when not grouping (all-connections view)", () => {
    expect(groupRows([port({ pid: 7, localPort: 1 }), port({ pid: 7, localPort: 1 })], false)).toHaveLength(2);
  });

  it("is localhost-only only if every merged socket is", () => {
    const rows = groupRows(
      [port({ pid: 7, localPort: 1, isLocalhostOnly: true }), port({ pid: 7, localPort: 1, localAddr: "0.0.0.0", isLocalhostOnly: false })],
      true,
    );
    expect(rows[0].isLocalhostOnly).toBe(false);
  });
});

describe("filterPorts", () => {
  const rows = [
    port({ localPort: 3000 }),
    port({ localPort: 135, processName: "svchost.exe", isSystem: true, isProtected: true, isDev: false }),
    port({ localPort: 5353, protocol: "UDP", state: "BOUND" }),
    port({ localPort: 50000, state: "ESTABLISHED", remoteAddr: "1.2.3.4", remotePort: 443 }),
  ];

  it("listen view hides system, UDP (by default) and non-listening sockets", () => {
    expect(filterPorts(rows, listen, "").map((r) => r.localPort)).toEqual([3000]);
  });

  it("shows UDP and system processes when toggled", () => {
    const shown = filterPorts(rows, { ...listen, showUdp: true, hideSystem: false }, "").map((r) => r.localPort);
    expect(shown).toEqual([3000, 135, 5353]);
  });

  it("all-connections view includes established sockets", () => {
    expect(filterPorts(rows, { ...listen, viewMode: "all" }, "").map((r) => r.localPort)).toContain(50000);
  });
});
