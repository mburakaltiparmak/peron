import { useCallback, useEffect, useRef, useState } from "react";
import { remoteList, toAppError } from "../api";
import type { AppError, Snapshot } from "../types";

/**
 * Ports of a remote host via `ssh … peron-cli list --json`. Polls only while mounted with a host
 * selected (the window is open) — never in the background, so the tray causes no SSH traffic.
 *
 * Every request carries a generation number; switching hosts bumps it, so a slow reply from the
 * previous host is dropped instead of being shown under the new one.
 */
export function useRemotePorts(hostId: string | null, intervalSecs: number, auto: boolean) {
  const [snapshot, setSnapshot] = useState<Snapshot | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<AppError | null>(null);
  const [updatedAt, setUpdatedAt] = useState<number | null>(null);
  const generation = useRef(0);
  const inFlight = useRef<number | null>(null);

  const refresh = useCallback(async () => {
    if (!hostId || inFlight.current === generation.current) return null;
    const gen = generation.current;
    inFlight.current = gen;
    setLoading(true);
    try {
      const s = await remoteList(hostId);
      if (gen !== generation.current) return null;
      setSnapshot(s);
      setError(null);
      setUpdatedAt(Date.now());
      return s.entries;
    } catch (e) {
      if (gen === generation.current) setError(toAppError(e));
      return null;
    } finally {
      if (gen === generation.current) {
        inFlight.current = null;
        setLoading(false);
      }
    }
  }, [hostId]);

  useEffect(() => {
    generation.current += 1;
    inFlight.current = null;
    setSnapshot(null);
    setError(null);
    setUpdatedAt(null);
    setLoading(false);
    if (!hostId) return;
    refresh();
    if (!auto) return;
    // An SSH round-trip per scan: never faster than every 10 s.
    const timer = setInterval(refresh, Math.max(intervalSecs, 10) * 1000);
    return () => clearInterval(timer);
  }, [hostId, intervalSecs, auto, refresh]);

  return { snapshot, loading, error, updatedAt, refresh };
}
