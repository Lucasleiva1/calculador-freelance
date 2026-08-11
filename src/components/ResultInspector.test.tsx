import { fireEvent, render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import type { MarketObservation, MarketOverview, QuoteService, ServiceResult } from "../domain/types";
import type { ProjectResult } from "../domain/quote";
import { ResultInspector } from "./ResultInspector";

describe("ResultInspector", () => {
  it("presenta el estado vacío sin precios ficticios", () => {
    render(<ResultInspector currency="USD" activeServiceId={null} suggestionsEnabled market={null} marketJob={null} onUpdateMarket={async () => undefined} onCancelMarket={async () => undefined} result={{ services: [], totalMinor: null, totalHours: 0, externalCostsMinor: 0, effectiveHourlyMinor: null, marginMicros: null, pricingTiers: { floorMinor: null, recommendedMinor: null, premiumMinor: null }, unpricedCount: 0, isPartial: false }} />);
    expect(screen.getByText("Agregá un servicio para comenzar la cotización.")).toBeInTheDocument();
    expect(screen.getAllByText("—").length).toBeGreaterThan(0);
    expect(screen.getByText("Todavía no hay datos")).toBeInTheDocument();
  });

  it("muestra la decisión de comparabilidad y conversión del snapshot", () => {
    const service: QuoteService = { id: "service", quoteId: "quote", serviceType: "video-editing", title: "Video", sortOrder: 0, configurationVersion: 1, configurationJson: "{}", calculatedSubtotalMinor: 54_000, suggestedSubtotalMinor: 65_000, finalSubtotalMinor: 72_000, hasOverride: true, manualSubtotalMinor: 72_000, manualReason: null, pricingSnapshotJson: null, serviceDefinitionVersion: 1, rowRevision: 0, deletedAt: null, createdAt: "", updatedAt: "" };
    const serviceResult: ServiceResult = { status: "ready", calculatedSubtotalMinor: 54_000, suggestedSubtotalMinor: 65_000, finalSubtotalMinor: 72_000, effectiveSubtotalMinor: 72_000, hasOverride: true, hours: 10, externalCostsMinor: 0, effectiveHourlyMinor: 7_200, appliedMarginMicros: null, lines: [], issues: [] };
    const result: ProjectResult = { services: [{ service, result: serviceResult }], totalMinor: 72_000, totalHours: 10, externalCostsMinor: 0, effectiveHourlyMinor: 7_200, marginMicros: 250_000, pricingTiers: { floorMinor: 54_000, recommendedMinor: 65_000, premiumMinor: 78_000 }, unpricedCount: 0, isPartial: false };
    const observation: MarketObservation = { id: "observation", sourceId: "source", sourceName: "Fuente AR", origin: "MANUAL", serviceType: "video-editing", subservice: "Edición", category: "Video", region: "AR", country: "Argentina", currency: "ARS", priceType: "PROJECT", unit: "por proyecto", priceMinMinor: null, priceMaxMinor: null, priceValueMinor: 800_000, originalValueText: "ARS 8.000", convertedValueMinor: 5_340, convertedCurrency: "USD", exchangeRateMicros: 14_985_000, exchangeRateDate: "2026-08-07", exchangeRateSource: "BCRA", experienceLevel: null, clientTier: null, sourceType: "professional_tariff", sourceUrl: "https://example.com/rate", publishedAt: "2026-08-01", retrievedAt: "2026-08-10T00:00:00Z", parserVersion: "test", confidence: "HIGH", comparisonEligibility: "ELIGIBLE", exclusionReason: null, rawFingerprint: "hash", evidenceSnippet: null, notes: null, createdAt: "2026-08-10T00:00:00Z", snapshotIncluded: false, snapshotExclusionReason: "La región no coincide con esta cotización.", snapshotNormalizedValueMinor: null };
    const snapshot = { id: "snapshot", quoteId: "quote", quoteServiceId: "service", queryContextJson: "{}", currency: "USD" as const, observationCount: 1, comparableObservationCount: 0, sourceCount: 0, minimumFilteredMinor: null, p25Minor: null, marketMedianMinor: null, p75Minor: null, maximumFilteredMinor: null, confidenceLevel: "INSUFFICIENT" as const, calculatedPriceMinor: 54_000, suggestedPriceMinor: null, finalPriceMinorAtCreation: 72_000, baseServiceRevision: 3, suggestionUpdateStatus: "INSUFFICIENT" as const, suggestionUpdateMessage: "No hay referencias comparables suficientes.", summaryJson: "{}", createdAt: "2026-08-10T00:00:00Z" };
    const market: MarketOverview = { latestSnapshot: snapshot, observations: [observation], history: [snapshot] };
    render(<ResultInspector currency="USD" activeServiceId="service" suggestionsEnabled market={market} marketJob={null} onUpdateMarket={async () => undefined} onCancelMarket={async () => undefined} result={result} />);
    fireEvent.click(screen.getByRole("button", { name: /ver fuentes/i }));
    expect(screen.getByText("La región no coincide con esta cotización.")).toBeInTheDocument();
    expect(screen.getByText(/Convertido:.*tasa 1\.498,5/i)).toBeInTheDocument();
    expect(screen.getByText("Propuesta de mercado")).toBeInTheDocument();
    expect(screen.getByText(/No hay referencias comparables suficientes\./)).toBeInTheDocument();
  });

  it("explica los requisitos pendientes y ofrece configurar la tarifa", () => {
    const onConfigureEconomy = vi.fn();
    const service: QuoteService = { id: "service", quoteId: "quote", serviceType: "video-editing", title: "Video", sortOrder: 0, configurationVersion: 1, configurationJson: "{}", calculatedSubtotalMinor: null, suggestedSubtotalMinor: null, finalSubtotalMinor: null, hasOverride: false, manualSubtotalMinor: null, manualReason: null, pricingSnapshotJson: null, serviceDefinitionVersion: 1, rowRevision: 0, deletedAt: null, createdAt: "", updatedAt: "" };
    const serviceResult: ServiceResult = { status: "incomplete", calculatedSubtotalMinor: null, suggestedSubtotalMinor: null, finalSubtotalMinor: null, effectiveSubtotalMinor: null, hasOverride: false, hours: 36, externalCostsMinor: 0, effectiveHourlyMinor: null, appliedMarginMicros: null, lines: [], issues: ["Configurá tu tarifa base en ARS."] };
    const result: ProjectResult = { services: [{ service, result: serviceResult }], totalMinor: null, totalHours: 36, externalCostsMinor: 0, effectiveHourlyMinor: null, marginMicros: null, pricingTiers: { floorMinor: null, recommendedMinor: null, premiumMinor: null }, unpricedCount: 1, isPartial: false };
    render(<ResultInspector currency="ARS" activeServiceId="service" suggestionsEnabled market={null} marketJob={null} onUpdateMarket={async () => undefined} onCancelMarket={async () => undefined} onConfigureEconomy={onConfigureEconomy} result={result} />);
    expect(screen.getByText("Faltan requisitos para calcular")).toBeInTheDocument();
    expect(screen.getByText("Configurá tu tarifa base en ARS.")).toBeInTheDocument();
    fireEvent.click(screen.getByRole("button", { name: /configurar tarifa/i }));
    expect(onConfigureEconomy).toHaveBeenCalledOnce();
  });
});
