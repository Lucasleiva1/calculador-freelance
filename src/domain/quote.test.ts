import { describe, expect, it } from "vitest";
import { evaluateWorkspace } from "./quote";
import { defaultVideoConfiguration } from "./video";
import type { AppSettings, PricingConfiguration, PricingEngine, QuoteService, ServiceDefinition, Workspace } from "./types";

const settings: AppSettings = { theme: "warm", hourlyRateArsMinor: null, hourlyRateUsdMinor: 5_000, usdToArsMicros: null, activeProjectId: null, suggestionsEnabled: true, suggestionStrategy: "balanced", baseCurrency: "USD", helpMode: "guided", localAiEnabled: false, ollamaBaseUrl: "http://127.0.0.1:11434", ollamaModel: null, aiAutoApplyHighConfidence: false, updatedAt: "2026-01-01" };
const pricing: PricingConfiguration = { definitions: [], parameters: [], options: [], rules: [], economicProfiles: [], marketSources: [], engineCategories: [], pricingEngines: [], engineSources: [] };
const service = (partial: Partial<QuoteService>): QuoteService => ({ id: "video", quoteId: "quote", serviceType: "video-editing", title: "Video", sortOrder: 0, configurationVersion: 1, configurationJson: JSON.stringify({ schemaVersion: 1, serviceType: "video-editing", data: { ...defaultVideoConfiguration(), estimatedHours: 2 } }), calculatedSubtotalMinor: 10_000, suggestedSubtotalMinor: null, finalSubtotalMinor: 10_000, hasOverride: false, manualSubtotalMinor: null, manualReason: null, pricingSnapshotJson: null, serviceDefinitionVersion: null, rowRevision: 0, deletedAt: null, createdAt: "", updatedAt: "", ...partial });
const workspace = (services: QuoteService[]): Workspace => ({ project: { id: "p", clientId: "c", clientName: "Cliente", name: "Proyecto", currency: "USD", marketScope: "argentina", status: "active", totalMinor: null, unpricedCount: 0, updatedAt: "" }, quote: { id: "quote", projectId: "p", version: 1, status: "draft", currency: "USD", notes: null, selectedPriceKind: "recommended", selectedPriceMinor: null, floorTotalMinor: null, recommendedTotalMinor: null, premiumTotalMinor: null, snapshotRevision: 0, savedAt: null, archivedAt: null, createdAt: "", updatedAt: "" }, services });

describe("resultado del proyecto", () => {
  it("marca el total como parcial si Programación todavía no tiene precio", () => {
    const programming = service({ id: "programming", serviceType: "programming", title: "Programación", configurationJson: JSON.stringify({ schemaVersion: 2, serviceType: "programming", data: { parameterValues: {}, externalCosts: [], notes: "" } }), calculatedSubtotalMinor: null, finalSubtotalMinor: null });
    const result = evaluateWorkspace(workspace([service({}), programming]), settings, pricing);
    expect(result.totalMinor).toBe(10_000);
    expect(result.isPartial).toBe(true);
    expect(result.unpricedCount).toBe(1);
  });

  it("no presenta cero como total de una cotización vacía", () => {
    expect(evaluateWorkspace(workspace([]), settings, pricing).totalMinor).toBeNull();
  });

  it("no presenta cero cuando todos los servicios siguen pendientes", () => {
    expect(evaluateWorkspace(workspace([service({ calculatedSubtotalMinor: null, finalSubtotalMinor: null })]), settings, pricing).totalMinor).toBeNull();
  });

  it("no reutiliza en Programación un cálculo manual viejo si esa profesión no tiene economía propia", () => {
    const definition: ServiceDefinition = { id: "service-programming", serviceType: "programming", name: "Programación", description: null, version: 1, enabled: true, suggestionsEnabled: true, defaultStrategy: "balanced", competitiveMarginMicros: null, balancedMarginMicros: null, premiumMarginMicros: null, createdAt: "", updatedAt: "" };
    const engine: PricingEngine = { id: "engine-programming", engineKey: "programming", name: "Programación", description: null, engineType: "service", categoryId: null, calculatorKey: "professional-service-v1", serviceDefinitionId: definition.id, unitKind: "hour", tagsJson: "[]", status: "active", classificationOrigin: "automatic", classificationConfidenceMicros: null, classificationExplanation: null, classificationVersion: 1, isSystem: true, createdAt: "", updatedAt: "", archivedAt: null };
    const programming = service({ id: "programming", serviceType: "programming", title: "Programación", configurationJson: JSON.stringify({ schemaVersion: 2, serviceType: "programming", data: { parameterValues: { estimatedHours: 8 }, externalCosts: [], notes: "" } }), calculatedSubtotalMinor: 99_000, suggestedSubtotalMinor: 99_000, finalSubtotalMinor: 99_000 });
    const separatedPricing: PricingConfiguration = { ...pricing, definitions: [definition], pricingEngines: [engine] };

    const result = evaluateWorkspace(workspace([programming]), settings, separatedPricing);

    expect(result.services[0].result.calculatedSubtotalMinor).toBeNull();
    expect(result.services[0].result.issues).toContain("Configurá tu economía en USD.");
    expect(result.totalMinor).toBeNull();
  });
});
