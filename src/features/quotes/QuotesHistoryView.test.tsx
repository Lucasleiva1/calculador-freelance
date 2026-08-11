import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import type { PricingConfiguration, QuoteHistoryDetail, QuoteHistoryItem, QuoteSnapshotDocument } from "../../domain/types";

const quote: QuoteHistoryItem = { id: "q", projectId: "p", projectName: "Campaña", clientId: "c", clientName: "ACME", currency: "USD", status: "draft", notes: "Entrega master", selectedPriceKind: "recommended", selectedPriceMinor: 120_000, floorTotalMinor: 90_000, recommendedTotalMinor: 120_000, premiumTotalMinor: 145_000, snapshotRevision: 1, savedAt: "2026-08-10T10:00:00Z", updatedAt: "2026-08-10T10:00:00Z", serviceCount: 1, serviceTitles: "Edición de video", serviceTypes: "video-editing" };
const snapshot: QuoteSnapshotDocument = { schemaVersion: 1, savedAt: quote.savedAt, revision: 1, quote: { id: "q", version: 1, status: "draft", currency: "USD", notes: "Entrega master", selectedPriceKind: "recommended", selectedPriceMinor: 120_000 }, project: { id: "p", name: "Campaña", marketScope: "international" }, client: { id: "c", name: "ACME", company: null }, services: [{ id: "s", serviceType: "video-editing", title: "Edición de video", sortOrder: 0, configurationVersion: 1, configuration: { estimatedHours: 10 }, calculatedSubtotalMinor: 90_000, suggestedSubtotalMinor: 120_000, finalSubtotalMinor: 120_000, hasOverride: false, manualSubtotalMinor: null, manualReason: null, serviceDefinitionVersion: 1, pricingSnapshot: {}, sources: { assigned: [{ id: "source", name: "Tarifario", url: "https://example.com", sourceType: "market", country: "AR", currency: "ARS", updatedAt: "2026-08-09", contribution: "Referencia por minuto", role: "reference", preference: "preferred" }], marketSnapshot: null, observations: [] } }], totals: { floorMinor: 90_000, recommendedMinor: 120_000, premiumMinor: 145_000, selectedMinor: 120_000, totalHoursMicros: 10_000_000, externalCostsMinor: 5_000, effectiveHourlyMinor: 11_500, marginMicros: 250_000 } };
const detail: QuoteHistoryDetail = { quote, snapshotJson: JSON.stringify(snapshot), snapshotCreatedAt: quote.savedAt, displayedRevision: 1, revisions: [{ revision: 1, reason: "manual_save", createdAt: quote.savedAt }] };
const apiMock = vi.hoisted(() => ({ listQuoteHistory: vi.fn(), getQuoteHistory: vi.fn(), updateQuoteAdmin: vi.fn(), duplicateQuote: vi.fn(), deleteQuotePermanently: vi.fn() }));
vi.mock("../../services/api", () => ({ api: apiMock }));

import { QuotesHistoryView } from "./QuotesHistoryView";

const pricing: PricingConfiguration = { definitions: [], parameters: [], options: [], rules: [], economicProfiles: [], marketSources: [], engineCategories: [], pricingEngines: [{ id: "engine", engineKey: "video-editing", name: "Edición de video", description: null, engineType: "service", categoryId: null, calculatorKey: "professional-service-v1", serviceDefinitionId: null, unitKind: "project", tagsJson: "[]", status: "active", classificationOrigin: "automatic", classificationConfidenceMicros: 1_000_000, classificationExplanation: null, classificationVersion: 1, isSystem: true, createdAt: "", updatedAt: "", archivedAt: null }], engineSources: [] };

describe("vista de cotizaciones", () => {
  beforeEach(() => { apiMock.listQuoteHistory.mockResolvedValue([quote]); apiMock.getQuoteHistory.mockResolvedValue(detail); });

  it("lista y abre el snapshot histórico con precios, módulos y fuentes", async () => {
    render(<QuotesHistoryView clients={[{ id: "c", name: "ACME", company: null, email: null, whatsapp: null, country: null, notes: null, status: "active", createdAt: "", updatedAt: "" }]} pricing={pricing} onOpenProject={async () => undefined} onDuplicated={() => undefined} />);
    expect(await screen.findByText("Campaña")).toBeInTheDocument();
    fireEvent.click(screen.getByRole("button", { name: /Campaña/i }));
    await waitFor(() => expect(apiMock.getQuoteHistory).toHaveBeenCalledWith("q", undefined));
    expect(await screen.findByText("Módulos congelados")).toBeInTheDocument();
    expect(screen.getByText("Tarifario")).toBeInTheDocument();
    expect(screen.getAllByText(/USD\s*1\.200/).length).toBeGreaterThan(0);
  });
});
