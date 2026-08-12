import { act, renderHook, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import type { QuoteService, SaveServiceInput } from "../domain/types";

const apiMock = vi.hoisted(() => ({ saveService: vi.fn() }));
vi.mock("../services/api", () => ({ api: apiMock }));

import { useAutosave } from "./useAutosave";

const input = (expectedRevision: number, title = "Diseño") : SaveServiceInput => ({
  id: "service-1",
  title,
  configurationVersion: 3,
  configurationJson: "{}",
  calculatedSubtotalMinor: null,
  suggestedSubtotalMinor: null,
  finalSubtotalMinor: null,
  hasOverride: false,
  manualSubtotalMinor: null,
  manualReason: null,
  pricingSnapshotJson: null,
  serviceDefinitionVersion: 2,
  expectedRevision,
});

const saved = (rowRevision: number, title = "Diseño"): QuoteService => ({
  ...input(rowRevision, title),
  quoteId: "quote-1",
  serviceType: "print-design",
  sortOrder: 0,
  rowRevision,
  deletedAt: null,
  createdAt: "2026-08-12T00:00:00Z",
  updatedAt: "2026-08-12T00:00:00Z",
});

describe("useAutosave", () => {
  beforeEach(() => apiMock.saveService.mockReset());

  it("uses the last confirmed revision even if the rendered service is briefly stale", async () => {
    apiMock.saveService
      .mockResolvedValueOnce(saved(1))
      .mockResolvedValueOnce(saved(2, "Diseño actualizado"));
    const onSaved = vi.fn();
    const { result } = renderHook(() => useAutosave(onSaved));

    act(() => result.current.schedule(input(0), true));
    await waitFor(() => expect(onSaved).toHaveBeenCalledTimes(1));

    act(() => result.current.schedule(input(0, "Diseño actualizado"), true));
    await waitFor(() => expect(apiMock.saveService).toHaveBeenCalledTimes(2));

    expect(apiMock.saveService.mock.calls[1][0].expectedRevision).toBe(1);
  });

  it("retries an errored queue entry when the app flushes before closing", async () => {
    apiMock.saveService
      .mockRejectedValueOnce(new Error("conflicto de revisión"))
      .mockResolvedValueOnce(saved(1));
    const { result } = renderHook(() => useAutosave(vi.fn()));

    act(() => result.current.schedule(input(0), true));
    await waitFor(() => expect(result.current.statuses["service-1"]).toBe("error"));

    let flushed = false;
    await act(async () => { flushed = await result.current.flushAll(); });

    expect(flushed).toBe(true);
    expect(apiMock.saveService).toHaveBeenCalledTimes(2);
    expect(result.current.statuses["service-1"]).toBe("saved");
  });

  it("does not resave a clean entry during a later calculation flush", async () => {
    apiMock.saveService.mockResolvedValueOnce(saved(1));
    const { result } = renderHook(() => useAutosave(vi.fn()));

    act(() => result.current.schedule(input(0), true));
    await waitFor(() => expect(result.current.statuses["service-1"]).toBe("saved"));

    let firstFlush = false;
    let secondFlush = false;
    await act(async () => { firstFlush = await result.current.flushAll(); });
    await act(async () => { secondFlush = await result.current.flushAll(); });

    expect(firstFlush).toBe(true);
    expect(secondFlush).toBe(true);
    expect(apiMock.saveService).toHaveBeenCalledTimes(1);
  });
});
