import { act, renderHook, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import type { Snapshot } from "../types";

const pending = new Map<string, { resolve: (s: Snapshot) => void; reject: (e: unknown) => void }[]>();

vi.mock("../api", () => ({
  remoteList: (id: string) =>
    new Promise<Snapshot>((resolve, reject) => {
      pending.set(id, [...(pending.get(id) ?? []), { resolve, reject }]);
    }),
  toAppError: (e: unknown) => ({ code: "remote", message: String(e) }),
}));

const { useRemotePorts } = await import("./useRemotePorts");

const snap = (host: string): Snapshot => ({
  schemaVersion: 1,
  tool: "peron-cli",
  version: "0.1.0-beta.1",
  host,
  os: "linux",
  generatedAtMs: 0,
  elevated: false,
  entries: [],
});

const reply = (id: string, s = snap(id)) => act(async () => pending.get(id)!.shift()!.resolve(s));

describe("useRemotePorts", () => {
  beforeEach(() => pending.clear());

  it("drops a late reply from the previous host after a fast A → B switch", async () => {
    const { result, rerender } = renderHook(({ id }) => useRemotePorts(id, 30, false), {
      initialProps: { id: "a" as string | null },
    });
    expect(pending.get("a")).toHaveLength(1);

    rerender({ id: "b" });
    // B's first request isn't blocked by A still being in flight.
    expect(pending.get("b")).toHaveLength(1);

    await reply("a");
    expect(result.current.snapshot).toBeNull();
    expect(result.current.loading).toBe(true);

    await reply("b");
    await waitFor(() => expect(result.current.snapshot?.host).toBe("b"));
    expect(result.current.loading).toBe(false);
  });

  it("ignores a late error from the previous host", async () => {
    const { result, rerender } = renderHook(({ id }) => useRemotePorts(id, 30, false), {
      initialProps: { id: "a" as string | null },
    });
    rerender({ id: "b" });
    await act(async () => pending.get("a")!.shift()!.reject("unreachable"));
    expect(result.current.error).toBeNull();
    await reply("b");
    expect(result.current.snapshot?.host).toBe("b");
  });

  it("clears state when the selection goes back to this computer", async () => {
    const { result, rerender } = renderHook(({ id }) => useRemotePorts(id, 30, false), {
      initialProps: { id: "a" as string | null },
    });
    await reply("a");
    expect(result.current.snapshot?.host).toBe("a");
    rerender({ id: null });
    expect(result.current.snapshot).toBeNull();
    expect(result.current.loading).toBe(false);
  });

  it("does not stack requests for the same host", async () => {
    const { result } = renderHook(() => useRemotePorts("a", 30, false));
    await act(async () => {
      await Promise.race([result.current.refresh(), Promise.resolve()]);
    });
    expect(pending.get("a")).toHaveLength(1);
    await reply("a");
    expect(result.current.loading).toBe(false);
  });
});
