import { useCallback, useEffect, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import { listPorts, PORTS_UPDATED, toAppError } from "../api";
import type { AppError, PortEntry } from "../types";

/** Port snapshot: fetched on demand, and pushed by the backend monitor while autoRefresh is on. */
export function usePorts(autoRefresh: boolean) {
  const [ports, setPorts] = useState<PortEntry[]>([]);
  const [updatedAt, setUpdatedAt] = useState<number | null>(null);
  const [loading, setLoading] = useState(false);
  const [loaded, setLoaded] = useState(false);
  const [error, setError] = useState<AppError | null>(null);

  const apply = useCallback((entries: PortEntry[]) => {
    setPorts(entries);
    setUpdatedAt(Date.now());
    setError(null);
    setLoaded(true);
  }, []);

  const refresh = useCallback(async () => {
    setLoading(true);
    try {
      const entries = await listPorts();
      apply(entries);
      return entries;
    } catch (e) {
      setError(toAppError(e));
      setLoaded(true);
      return null;
    } finally {
      setLoading(false);
    }
  }, [apply]);

  useEffect(() => {
    refresh();
  }, [refresh]);

  useEffect(() => {
    if (!autoRefresh) return;
    const unlisten = listen<PortEntry[]>(PORTS_UPDATED, (e) => apply(e.payload));
    return () => {
      unlisten.then((f) => f());
    };
  }, [autoRefresh, apply]);

  return { ports, updatedAt, loading, loaded, error, refresh };
}

/** Current time, re-rendered every `intervalMs` so durations stay fresh without a rescan. */
export function useNow(intervalMs = 30_000) {
  const [now, setNow] = useState(Date.now());
  useEffect(() => {
    const t = setInterval(() => setNow(Date.now()), intervalMs);
    return () => clearInterval(t);
  }, [intervalMs]);
  return now;
}
