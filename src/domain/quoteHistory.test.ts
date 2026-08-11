import { describe, expect, it } from "vitest";
import type { QuoteHistoryItem } from "./types";
import { filterQuoteHistory, parseQuoteSnapshot } from "./quoteHistory";

const item = (partial: Partial<QuoteHistoryItem>): QuoteHistoryItem => ({
  id: "q1", projectId: "p1", projectName: "Campaña", clientId: "c1", clientName: "ACME",
  currency: "USD", status: "draft", notes: null, selectedPriceKind: "recommended",
  selectedPriceMinor: 120_000, floorTotalMinor: 90_000, recommendedTotalMinor: 120_000,
  premiumTotalMinor: 145_000, snapshotRevision: 1, savedAt: "2026-08-10T10:00:00Z",
  updatedAt: "2026-08-10T10:00:00Z", serviceCount: 1, serviceTitles: "Video",
  serviceTypes: "video-editing", ...partial,
});

describe("historial de cotizaciones", () => {
  it("busca, filtra y ordena sin perder la lista original", () => {
    const input = [item({}), item({ id: "q2", projectName: "Sitio", clientName: "Beta", currency: "ARS", selectedPriceMinor: 80_000, savedAt: "2026-08-09T10:00:00Z", serviceTypes: "programming" })];
    const result = filterQuoteHistory(input, { query: "beta", status: "all", serviceType: "programming", currency: "ARS", sort: "price-desc" });
    expect(result.map((quote) => quote.id)).toEqual(["q2"]);
    expect(input.map((quote) => quote.id)).toEqual(["q1", "q2"]);
  });

  it("rechaza snapshots rotos sin hacer fallar la vista", () => {
    expect(parseQuoteSnapshot("no-es-json")).toBeNull();
    expect(parseQuoteSnapshot('{"schemaVersion":1,"services":[]}')?.services).toEqual([]);
  });
});
