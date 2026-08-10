import { useCallback, useEffect, useRef, useState } from "react";
import type { QuoteService, SaveServiceInput, SaveStatus } from "../domain/types";
import { api } from "../services/api";

interface QueueEntry {
  latest: SaveServiceInput;
  timer: ReturnType<typeof setTimeout> | null;
  saving: boolean;
  waiters: Array<(ok: boolean) => void>;
}

export function useAutosave(onSaved: (service: QuoteService) => void) {
  const queue = useRef(new Map<string, QueueEntry>());
  const [statuses, setStatuses] = useState<Record<string, SaveStatus>>({});
  const [errors, setErrors] = useState<Record<string, string>>({});
  const alive = useRef(true);
  const runRef = useRef<(id: string) => Promise<void>>(async () => undefined);

  useEffect(() => () => { alive.current = false; }, []);

  const setStatus = useCallback((id: string, status: SaveStatus, error?: string) => {
    if (!alive.current) return;
    setStatuses((current) => ({ ...current, [id]: status }));
    setErrors((current) => {
      const next = { ...current };
      if (error) next[id] = error;
      else delete next[id];
      return next;
    });
  }, []);

  const run = useCallback(async (id: string) => {
    const entry = queue.current.get(id);
    if (!entry || entry.saving) return;
    if (entry.timer) clearTimeout(entry.timer);
    entry.timer = null;
    entry.saving = true;
    const payload = entry.latest;
    setStatus(id, "saving");
    try {
      const saved = await api.saveService(payload);
      onSaved(saved);
      entry.saving = false;
      if (entry.latest !== payload) {
        entry.latest = { ...entry.latest, expectedRevision: saved.rowRevision };
        void runRef.current(id);
      } else {
        setStatus(id, "saved");
        entry.waiters.splice(0).forEach((resolve) => resolve(true));
      }
    } catch (error) {
      entry.saving = false;
      setStatus(id, "error", String(error));
      entry.waiters.splice(0).forEach((resolve) => resolve(false));
    }
  }, [onSaved, setStatus]);

  useEffect(() => { runRef.current = run; }, [run]);

  const schedule = useCallback((input: SaveServiceInput, immediate = false) => {
    const existing = queue.current.get(input.id);
    const entry: QueueEntry = existing ?? { latest: input, timer: null, saving: false, waiters: [] };
    entry.latest = input;
    if (entry.timer) clearTimeout(entry.timer);
    queue.current.set(input.id, entry);
    setStatus(input.id, "saving");
    if (immediate && !entry.saving) void run(input.id);
    else entry.timer = setTimeout(() => void run(input.id), 600);
  }, [run, setStatus]);

  const retry = useCallback((id: string) => void run(id), [run]);

  const flushAll = useCallback(async (): Promise<boolean> => {
    const waits = [...queue.current.entries()].map(([id, entry]) => {
      if (!entry.timer && !entry.saving && statuses[id] !== "saving") return Promise.resolve(statuses[id] !== "error");
      return new Promise<boolean>((resolve) => {
        entry.waiters.push(resolve);
        if (!entry.saving) void run(id);
      });
    });
    const results = await Promise.all(waits);
    return results.every(Boolean);
  }, [run, statuses]);

  return { statuses, errors, schedule, retry, flushAll };
}
