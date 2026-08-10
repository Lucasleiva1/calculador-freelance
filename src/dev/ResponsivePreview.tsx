import { useEffect, useState } from "react";
import { Sidebar } from "../components/Sidebar";
import { Topbar } from "../components/Topbar";
import type { AppSettings, PricingConfiguration, ProjectSummary, QuoteService, ServiceConfigurationEnvelope, Workspace } from "../domain/types";
import { calculateProduct, type ProductConfiguration } from "../domain/product";
import type { ProjectResult } from "../domain/quote";
import { WorkspaceView } from "../features/quotes/WorkspaceView";

const timestamp = "2026-08-10T20:00:00Z";
const project: ProjectSummary = { id: "preview-project", clientId: "preview-client", clientName: "Cliente", name: "Venta de remeras", currency: "USD", marketScope: "both", status: "active", totalMinor: 45_000, unpricedCount: 0, updatedAt: timestamp };
const config: ProductConfiguration = {
  quantity: 20,
  costs: [
    { id: "shirt", name: "Remera base", amountMinor: 800, currency: "USD", scope: "per_unit", category: "material" },
    { id: "print", name: "Estampado DTF", amountMinor: 350, currency: "USD", scope: "per_unit", category: "production" },
    { id: "packaging", name: "Packaging", amountMinor: 120, currency: "USD", scope: "per_unit", category: "packaging" },
  ],
  wastePercent: 5, commissionPercent: 10, taxPercent: 5,
  recommendedMarginPercent: 30, premiumMarginPercent: 45, selectedTier: "recommended",
};
const envelope: ServiceConfigurationEnvelope<ProductConfiguration> = { schemaVersion: 1, serviceType: "venta-remeras", data: config };
const service: QuoteService = { id: "preview-service", quoteId: "preview-quote", serviceType: "venta-remeras", title: "Venta de remeras estampadas", sortOrder: 0, configurationVersion: 1, configurationJson: JSON.stringify(envelope), calculatedSubtotalMinor: 31_377, suggestedSubtotalMinor: 48_491, finalSubtotalMinor: 48_491, hasOverride: false, manualSubtotalMinor: null, manualReason: null, pricingSnapshotJson: null, serviceDefinitionVersion: 1, rowRevision: 1, deletedAt: null, createdAt: timestamp, updatedAt: timestamp };
const workspace: Workspace = { project, quote: { id: "preview-quote", projectId: project.id, version: 1, status: "draft", currency: "USD", createdAt: timestamp, updatedAt: timestamp }, services: [service] };
const settings: AppSettings = { theme: "warm", hourlyRateArsMinor: 1_500_000, hourlyRateUsdMinor: 2_500, usdToArsMicros: 13_200_000, activeProjectId: project.id, suggestionsEnabled: true, suggestionStrategy: "balanced", baseCurrency: "USD", helpMode: "guided", localAiEnabled: false, ollamaBaseUrl: "http://127.0.0.1:11434", ollamaModel: null, aiAutoApplyHighConfidence: false, updatedAt: timestamp };
const pricing: PricingConfiguration = {
  definitions: [], parameters: [], options: [], rules: [], economicProfiles: [], marketSources: [], engineCategories: [], engineSources: [],
  pricingEngines: [{ id: "preview-engine", engineKey: "venta-remeras", name: "Venta de remeras", description: "Producto físico de indumentaria", engineType: "product", categoryId: "category-apparel", calculatorKey: "physical-product-v1", serviceDefinitionId: null, unitKind: "unit", tagsJson: '["remeras","indumentaria"]', status: "active", classificationOrigin: "automatic", classificationConfidenceMicros: 930_000, classificationExplanation: "Producto físico por unidad.", classificationVersion: 1, isSystem: false, createdAt: timestamp, updatedAt: timestamp, archivedAt: null }],
};

export function ResponsivePreview() {
  const [productConfig, setProductConfig] = useState(config);
  const result = calculateProduct(productConfig, { currency: "USD", hourlyRateMinor: 2_500, usdToArsMicros: 13_200_000 });
  const currentService = { ...service, configurationJson: JSON.stringify({ ...envelope, data: productConfig }) };
  const currentWorkspace = { ...workspace, services: [currentService] };
  const projectResult: ProjectResult = { services: [{ service: currentService, result }], totalMinor: result.effectiveSubtotalMinor, totalHours: 0, externalCostsMinor: result.externalCostsMinor, effectiveHourlyMinor: null, unpricedCount: result.status === "ready" ? 0 : 1, isPartial: result.status !== "ready" };
  useEffect(() => { document.documentElement.dataset.theme = "warm"; document.documentElement.dataset.help = "guided"; }, []);
  return <div className="app-shell">
    <Sidebar section="workspace" onSection={() => undefined} onNewProject={() => undefined} />
    <div className="app-body">
      <Topbar project={project} projects={[project]} theme="warm" usdToArsMicros={settings.usdToArsMicros} onProject={() => undefined} onNewProject={() => undefined} onCurrency={() => undefined} onToggleTheme={() => undefined} onSettings={() => undefined} />
      <WorkspaceView workspace={currentWorkspace} settings={settings} pricing={pricing} result={projectResult} presets={[]} statuses={{ [service.id]: "saved" }} errors={{}} activeServiceId={service.id} onActiveService={() => undefined} onAddService={async () => undefined} onVideoChange={() => undefined} onProgrammingChange={() => undefined} onGenericEngineChange={(_, next) => setProductConfig(next as ProductConfiguration)} onFinalPriceChange={() => undefined} onTitleChange={() => undefined} onDeleteService={async () => undefined} onMoveService={async () => undefined} onRetry={() => undefined} onSavePreset={async () => undefined} onUpdatePreset={async () => undefined} onDeletePreset={async () => undefined} onRestorePreset={async () => undefined} market={null} marketJob={null} onUpdateMarket={async () => undefined} onCancelMarket={async () => undefined} />
    </div>
  </div>;
}
