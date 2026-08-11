import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import type { AppSettings, PricingConfiguration, QuoteService, Workspace } from "../../domain/types";
import type { ProjectResult } from "../../domain/quote";
import { defaultVideoConfiguration } from "../../domain/video";
import { WorkspaceView } from "./WorkspaceView";

const stamp = "2026-08-11T12:00:00Z";
const service: QuoteService = {
  id: "video", quoteId: "quote", serviceType: "video-editing", title: "Edición de video", sortOrder: 0,
  configurationVersion: 1, configurationJson: JSON.stringify({ schemaVersion: 1, serviceType: "video-editing", data: { ...defaultVideoConfiguration(), estimatedHours: 2 } }),
  calculatedSubtotalMinor: null, suggestedSubtotalMinor: null, finalSubtotalMinor: null, hasOverride: false,
  manualSubtotalMinor: null, manualReason: null, pricingSnapshotJson: null, serviceDefinitionVersion: 1,
  rowRevision: 1, deletedAt: null, createdAt: stamp, updatedAt: stamp,
};
const workspace: Workspace = {
  project: { id: "project", clientId: "client", clientName: "Cliente", name: "Proyecto", currency: "ARS", marketScope: "argentina", status: "active", totalMinor: null, unpricedCount: 1, updatedAt: stamp },
  quote: { id: "quote", projectId: "project", version: 1, status: "draft", currency: "ARS", notes: null, selectedPriceKind: "recommended", selectedPriceMinor: null, floorTotalMinor: null, recommendedTotalMinor: null, premiumTotalMinor: null, snapshotRevision: 0, savedAt: null, archivedAt: null, createdAt: stamp, updatedAt: stamp },
  services: [service],
};
const settings: AppSettings = { theme: "warm", hourlyRateArsMinor: null, hourlyRateUsdMinor: 2_500, usdToArsMicros: null, activeProjectId: "project", suggestionsEnabled: true, suggestionStrategy: "balanced", baseCurrency: "ARS", helpMode: "guided", localAiEnabled: false, ollamaBaseUrl: "http://127.0.0.1:11434", ollamaModel: null, aiAutoApplyHighConfidence: false, updatedAt: stamp };
const pricing: PricingConfiguration = { definitions: [], parameters: [], options: [], rules: [], economicProfiles: [], marketSources: [], engineCategories: [], engineSources: [], pricingEngines: [] };
const result: ProjectResult = {
  services: [{ service, result: { status: "incomplete", calculatedSubtotalMinor: null, suggestedSubtotalMinor: null, finalSubtotalMinor: null, effectiveSubtotalMinor: null, hasOverride: false, hours: 2, externalCostsMinor: 0, effectiveHourlyMinor: null, appliedMarginMicros: null, lines: [], issues: ["Configurá tu tarifa base en ARS."] } }],
  totalMinor: null, totalHours: 2, externalCostsMinor: 0, effectiveHourlyMinor: null, marginMicros: null,
  pricingTiers: { floorMinor: null, recommendedMinor: null, premiumMinor: null }, unpricedCount: 1, isPartial: false,
};

describe("WorkspaceView", () => {
  it("protege los parámetros del módulo durante una actualización de mercado", () => {
    render(<WorkspaceView workspace={workspace} settings={settings} pricing={pricing} result={result} presets={[]} statuses={{ video: "saved" }} errors={{}} activeServiceId="video" onActiveService={() => undefined} onAddService={async () => undefined} onVideoChange={() => undefined} onProgrammingChange={() => undefined} onGenericEngineChange={() => undefined} onFinalPriceChange={() => undefined} onTitleChange={() => undefined} onDeleteService={async () => undefined} onMoveService={async () => undefined} onRetry={() => undefined} onSavePreset={async () => undefined} onUpdatePreset={async () => undefined} onDeletePreset={async () => undefined} onRestorePreset={async () => undefined} market={null} marketJob={null} onUpdateMarket={async () => undefined} onCancelMarket={async () => undefined} onSaveQuote={async () => undefined} marketUpdating />);

    expect(screen.getByText(/Tus parámetros quedan bloqueados/i)).toBeInTheDocument();
    expect(screen.getByLabelText("Tiempo estimado")).toBeDisabled();
    for (const button of screen.getAllByRole("button", { name: /calcular (?:y actualizar fuentes|automático)/i })) expect(button).toBeDisabled();
  });
});
