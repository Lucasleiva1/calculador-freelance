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
    expect(screen.getByText("Todavía no hay datos automáticos.")).toBeInTheDocument();
  });

  it("muestra la decisión de comparabilidad y conversión del snapshot", () => {
    const service: QuoteService = { id: "service", quoteId: "quote", serviceType: "video-editing", title: "Video", sortOrder: 0, configurationVersion: 1, configurationJson: "{}", calculatedSubtotalMinor: 54_000, suggestedSubtotalMinor: 65_000, finalSubtotalMinor: 72_000, hasOverride: true, manualSubtotalMinor: 72_000, manualReason: null, pricingSnapshotJson: null, serviceDefinitionVersion: 1, rowRevision: 0, deletedAt: null, createdAt: "", updatedAt: "" };
    const serviceResult: ServiceResult = { status: "ready", calculatedSubtotalMinor: 54_000, suggestedSubtotalMinor: 65_000, finalSubtotalMinor: 72_000, effectiveSubtotalMinor: 72_000, hasOverride: true, hours: 10, externalCostsMinor: 0, effectiveHourlyMinor: 7_200, appliedMarginMicros: null, lines: [], issues: [] };
    const result: ProjectResult = { services: [{ service, result: serviceResult }], totalMinor: 72_000, totalHours: 10, externalCostsMinor: 0, effectiveHourlyMinor: 7_200, marginMicros: 250_000, pricingTiers: { floorMinor: 54_000, recommendedMinor: 65_000, premiumMinor: 78_000 }, unpricedCount: 0, isPartial: false };
    const observation: MarketObservation = { id: "observation", sourceId: "source", sourceName: "Fuente AR", origin: "MANUAL", serviceType: "video-editing", subservice: "Edición", category: "Video", region: "AR", country: "Argentina", currency: "ARS", priceType: "PROJECT", unit: "por proyecto", priceMinMinor: null, priceMaxMinor: null, priceValueMinor: 800_000, originalValueText: "ARS 8.000", convertedValueMinor: 5_340, convertedCurrency: "USD", exchangeRateMicros: 14_985_000, exchangeRateDate: "2026-08-07", exchangeRateSource: "BCRA", experienceLevel: null, clientTier: null, sourceType: "professional_tariff", sourceUrl: "https://example.com/rate", publishedAt: "2026-08-01", retrievedAt: "2026-08-10T00:00:00Z", parserVersion: "test", confidence: "HIGH", comparisonEligibility: "ELIGIBLE", exclusionReason: null, rawFingerprint: "hash", evidenceSnippet: null, notes: null, createdAt: "2026-08-10T00:00:00Z", snapshotIncluded: false, snapshotExclusionReason: "La región no coincide con esta cotización.", snapshotNormalizedValueMinor: null };
    const automaticSummary = { minimumFilteredMinor: 50_000, p25Minor: 55_000, medianMinor: 60_000, p75Minor: 65_000, maximumFilteredMinor: 70_000, confidenceLevel: "LOW", comparableCount: 2, sourceCount: 1, recentCount: 2, salaryExcludedCount: 0, explanations: [] };
    const snapshot = { id: "snapshot", quoteId: "quote", quoteServiceId: "service", queryContextJson: "{}", currency: "USD" as const, observationCount: 1, comparableObservationCount: 0, sourceCount: 0, minimumFilteredMinor: null, p25Minor: null, marketMedianMinor: null, p75Minor: null, maximumFilteredMinor: null, confidenceLevel: "INSUFFICIENT" as const, calculatedPriceMinor: 54_000, suggestedPriceMinor: null, finalPriceMinorAtCreation: 72_000, baseServiceRevision: 3, suggestionUpdateStatus: "INSUFFICIENT" as const, suggestionUpdateMessage: "No hay referencias comparables suficientes.", summaryJson: JSON.stringify({ pricingOptions: { market: { summary: automaticSummary, suggestedPriceMinor: 60_000, region: "AR" }, international: { summary: automaticSummary, suggestedPriceMinor: 90_000, region: "INTERNATIONAL" } }, fxRateMicros: 14_985_000, fxRateDate: "2026-08-07", fxRateSource: "BCRA" }), createdAt: "2026-08-10T00:00:00Z" };
    const market: MarketOverview = { latestSnapshot: snapshot, observations: [observation], history: [snapshot] };
    render(<ResultInspector currency="USD" activeServiceId="service" suggestionsEnabled market={market} marketJob={null} onUpdateMarket={async () => undefined} onCancelMarket={async () => undefined} result={result} />);
    fireEvent.click(screen.getByRole("button", { name: /ver fuentes/i }));
    expect(screen.getByText("La región no coincide con esta cotización.")).toBeInTheDocument();
    expect(screen.getByText(/Convertido:.*tasa 1\.498,5/i)).toBeInTheDocument();
    expect(screen.getByText("Sostenible")).toBeInTheDocument();
    expect(screen.getByText("Mercado")).toBeInTheDocument();
    expect(screen.getByText("Internacional")).toBeInTheDocument();
    expect(screen.getByRole("button", { name: /ver en ars/i })).toBeInTheDocument();
  });

  it("explica los requisitos pendientes y ofrece configurar la tarifa", () => {
    const onConfigureEconomy = vi.fn();
    const service: QuoteService = { id: "service", quoteId: "quote", serviceType: "video-editing", title: "Video", sortOrder: 0, configurationVersion: 1, configurationJson: "{}", calculatedSubtotalMinor: null, suggestedSubtotalMinor: null, finalSubtotalMinor: null, hasOverride: false, manualSubtotalMinor: null, manualReason: null, pricingSnapshotJson: null, serviceDefinitionVersion: 1, rowRevision: 0, deletedAt: null, createdAt: "", updatedAt: "" };
    const serviceResult: ServiceResult = { status: "incomplete", calculatedSubtotalMinor: null, suggestedSubtotalMinor: null, finalSubtotalMinor: null, effectiveSubtotalMinor: null, hasOverride: false, hours: 36, externalCostsMinor: 0, effectiveHourlyMinor: null, appliedMarginMicros: null, lines: [], issues: ["Configurá tu tarifa base en ARS."] };
    const result: ProjectResult = { services: [{ service, result: serviceResult }], totalMinor: null, totalHours: 36, externalCostsMinor: 0, effectiveHourlyMinor: null, marginMicros: null, pricingTiers: { floorMinor: null, recommendedMinor: null, premiumMinor: null }, unpricedCount: 1, isPartial: false };
    render(<ResultInspector currency="ARS" activeServiceId="service" suggestionsEnabled market={null} marketJob={null} onUpdateMarket={async () => undefined} onCancelMarket={async () => undefined} onConfigureEconomy={onConfigureEconomy} result={result} />);
    expect(screen.getByText("El precio local está pendiente")).toBeInTheDocument();
    expect(screen.getAllByText("Configurá tu tarifa base en ARS.").length).toBeGreaterThan(0);
    fireEvent.click(screen.getByRole("button", { name: /completar datos/i }));
    expect(onConfigureEconomy).toHaveBeenCalledOnce();
  });

  it("no culpa a la economía cuando falta un dato del alcance", () => {
    const service: QuoteService = { id: "service", quoteId: "quote", serviceType: "print-design", title: "Estampa", sortOrder: 0, configurationVersion: 2, configurationJson: "{}", calculatedSubtotalMinor: null, suggestedSubtotalMinor: null, finalSubtotalMinor: null, hasOverride: false, manualSubtotalMinor: null, manualReason: null, pricingSnapshotJson: null, serviceDefinitionVersion: 1, rowRevision: 0, deletedAt: null, createdAt: "", updatedAt: "" };
    const serviceResult: ServiceResult = { status: "incomplete", calculatedSubtotalMinor: null, suggestedSubtotalMinor: null, finalSubtotalMinor: null, effectiveSubtotalMinor: null, hasOverride: false, hours: 14.75, externalCostsMinor: 0, effectiveHourlyMinor: null, appliedMarginMicros: null, lines: [], issues: ["Completá “Origen del diseño”."] };
    const result: ProjectResult = { services: [{ service, result: serviceResult }], totalMinor: null, totalHours: 14.75, externalCostsMinor: 0, effectiveHourlyMinor: null, marginMicros: null, pricingTiers: { floorMinor: null, recommendedMinor: null, premiumMinor: null }, unpricedCount: 1, isPartial: false };
    render(<ResultInspector currency="ARS" activeServiceId="service" suggestionsEnabled market={null} marketJob={null} onUpdateMarket={async () => undefined} onCancelMarket={async () => undefined} onConfigureEconomy={vi.fn()} result={result} />);
    expect(screen.getAllByText("Completá “Origen del diseño”.").length).toBeGreaterThan(0);
    expect(screen.queryByRole("button", { name: /completar datos/i })).not.toBeInTheDocument();
  });

  it("en Estampas guarda una elección sostenible explícita y no ofrece un cuarto precio manual", () => {
    const onFinalPriceChange = vi.fn();
    const service: QuoteService = { id: "print", quoteId: "quote", serviceType: "print-design", title: "Estampa", sortOrder: 0, configurationVersion: 3, configurationJson: "{}", calculatedSubtotalMinor: 50_000, suggestedSubtotalMinor: null, finalSubtotalMinor: null, hasOverride: false, manualSubtotalMinor: null, manualReason: null, pricingSnapshotJson: null, serviceDefinitionVersion: 2, rowRevision: 0, deletedAt: null, createdAt: "", updatedAt: "" };
    const serviceResult: ServiceResult = { status: "ready", calculatedSubtotalMinor: 50_000, suggestedSubtotalMinor: null, finalSubtotalMinor: null, effectiveSubtotalMinor: null, hasOverride: false, hours: 2, externalCostsMinor: 0, effectiveHourlyMinor: null, appliedMarginMicros: null, lines: [], issues: [] };
    const result: ProjectResult = { services: [{ service, result: serviceResult }], totalMinor: null, totalHours: 2, externalCostsMinor: 0, effectiveHourlyMinor: null, marginMicros: null, pricingTiers: { floorMinor: null, recommendedMinor: null, premiumMinor: null }, unpricedCount: 1, isPartial: false };
    render(<ResultInspector currency="ARS" activeServiceId="print" suggestionsEnabled market={null} marketJob={null} onUpdateMarket={async () => undefined} onCancelMarket={async () => undefined} onFinalPriceChange={onFinalPriceChange} result={result} />);

    fireEvent.click(screen.getByRole("button", { name: "Usar este precio" }));

    expect(onFinalPriceChange).toHaveBeenCalledWith(50_000, "Elegido desde Precio sostenible", expect.objectContaining({ kind: "sustainable", amountMinor: 50_000, currency: "ARS", marketSnapshotId: null }));
    expect(screen.queryByText("Ajustar el precio final")).not.toBeInTheDocument();
  });
});
