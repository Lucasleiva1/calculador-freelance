import { describe, expect, it } from "vitest";
import { buildMarketQueryContext, suggestedFromSnapshot } from "./market";
import type { MarketSnapshot, QuoteService } from "./types";

const service: QuoteService = {
  id: "service", quoteId: "quote", serviceType: "video-editing", title: "Video", sortOrder: 0,
  configurationVersion: 1, configurationJson: JSON.stringify({ data: { pieceType: "youtube", finalDuration: "10:30", quantity: 2, estimatedHours: 8, subtitles: "designed", privateClientName: "No debe salir" } }),
  calculatedSubtotalMinor: 50_000, suggestedSubtotalMinor: 55_000, finalSubtotalMinor: 72_000,
  hasOverride: true, manualSubtotalMinor: 72_000, manualReason: "Valor acordado", pricingSnapshotJson: null,
  serviceDefinitionVersion: 1, rowRevision: 0, deletedAt: null, createdAt: "", updatedAt: "",
};

const snapshot: MarketSnapshot = {
  id: "snapshot", quoteId: "quote", quoteServiceId: "service", queryContextJson: "{}", currency: "USD",
  observationCount: 8, comparableObservationCount: 6, sourceCount: 3, minimumFilteredMinor: 58_000,
  p25Minor: 60_000, marketMedianMinor: 67_000, p75Minor: 79_000, maximumFilteredMinor: 90_000,
  confidenceLevel: "MEDIUM", calculatedPriceMinor: 54_000, suggestedPriceMinor: 65_000,
  finalPriceMinorAtCreation: 72_000, summaryJson: "{}", createdAt: "2026-08-10T12:00:00Z",
};

describe("market intelligence domain", () => {
  it("construye contexto abstracto y no copia datos privados arbitrarios", () => {
    const context = buildMarketQueryContext(service, "both");
    expect(context.durationMinutes).toBe(10.5);
    expect(context.subtype).toBe("youtube");
    expect(JSON.stringify(context)).not.toContain("No debe salir");
    expect(JSON.stringify(context)).not.toContain("privateClientName");
  });

  it("aplica estrategia de mercado sin tocar el precio final capturado", () => {
    expect(suggestedFromSnapshot(snapshot, "balanced")).toBe(61_800);
    expect(snapshot.finalPriceMinorAtCreation).toBe(72_000);
  });

  it("no inventa sugerencia cuando el mercado es insuficiente", () => {
    expect(suggestedFromSnapshot({ ...snapshot, confidenceLevel: "INSUFFICIENT" }, "premium")).toBeNull();
  });
});
