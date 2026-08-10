import { describe, expect, it } from "vitest";
import { evaluateWorkspace } from "./quote";
import { defaultVideoConfiguration } from "./video";
import type { AppSettings, PricingConfiguration, QuoteService, Workspace } from "./types";

const settings: AppSettings = { theme: "warm", hourlyRateArsMinor: null, hourlyRateUsdMinor: 5_000, usdToArsMicros: null, activeProjectId: null, suggestionsEnabled: true, suggestionStrategy: "balanced", baseCurrency: "USD", helpMode: "guided", localAiEnabled: false, ollamaBaseUrl: "http://127.0.0.1:11434", ollamaModel: null, aiAutoApplyHighConfidence: false, updatedAt: "2026-01-01" };
const pricing: PricingConfiguration = { definitions: [], parameters: [], options: [], rules: [], economicProfiles: [], marketSources: [], engineCategories: [], pricingEngines: [], engineSources: [] };
const service = (partial: Partial<QuoteService>): QuoteService => ({ id: "video", quoteId: "quote", serviceType: "video-editing", title: "Video", sortOrder: 0, configurationVersion: 1, configurationJson: JSON.stringify({ schemaVersion: 1, serviceType: "video-editing", data: { ...defaultVideoConfiguration(), estimatedHours: 2 } }), calculatedSubtotalMinor: 10_000, suggestedSubtotalMinor: null, finalSubtotalMinor: 10_000, hasOverride: false, manualSubtotalMinor: null, manualReason: null, pricingSnapshotJson: null, serviceDefinitionVersion: null, rowRevision: 0, deletedAt: null, createdAt: "", updatedAt: "", ...partial });
const workspace = (services: QuoteService[]): Workspace => ({ project: { id: "p", clientId: "c", clientName: "Cliente", name: "Proyecto", currency: "USD", marketScope: "argentina", status: "active", totalMinor: null, unpricedCount: 0, updatedAt: "" }, quote: { id: "quote", projectId: "p", version: 1, status: "draft", currency: "USD", createdAt: "", updatedAt: "" }, services });

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
});
