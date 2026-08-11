import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";
import type {
  Bootstrap,
  Client,
  ClientInput,
  CreateProjectInput,
  Currency,
  Preset,
  ProjectSummary,
  QuoteService,
  ServiceConfigurationEnvelope,
  ServiceType,
  SettingsInput,
  Workspace,
  PricingConfiguration,
  MarketOverview,
  MarketResearchJob,
  SaveServiceInput,
} from "../domain/types";
import { parseVideoEnvelope, type VideoConfiguration } from "../domain/video";
import { parseProgrammingEnvelope, type ProgrammingConfiguration } from "../domain/programming";
import type { HybridConfiguration, ProductConfiguration } from "../domain/product";
import { calculateHybrid, calculateProduct } from "../domain/product";
import { evaluateWorkspace } from "../domain/quote";
import { activeHourlyRate, createPricingSnapshot, runPricingEngine } from "../domain/pricingEngine";
import { api } from "../services/api";
import { useAutosave } from "../hooks/useAutosave";
import { Sidebar, type AppSection } from "../components/Sidebar";
import { Topbar } from "../components/Topbar";
import { NewProjectModal } from "../components/NewProjectModal";
import { EmptyState, Button } from "../components/ui";
import { WorkspaceView } from "../features/quotes/WorkspaceView";
import { ClientsView } from "../features/clients/ClientsView";
import { ProjectsView } from "../features/projects/ProjectsView";
import { SettingsView, type SettingsTab } from "../features/settings/SettingsView";
import { MarketView } from "../features/market/MarketView";
import { SaveQuoteModal } from "../features/quotes/SaveQuoteModal";
import { QuotesHistoryView } from "../features/quotes/QuotesHistoryView";
import { ClientDocumentModal } from "../features/quotes/ClientDocumentModal";

function presetConfiguration(config: VideoConfiguration) {
  return JSON.stringify(Object.fromEntries(Object.entries(config).filter(([key]) => !["effortAmount", "effortUnit", "hoursPerDay", "estimatedHours", "externalCosts", "urgencyFeeMinor"].includes(key))));
}

export function App() {
  const [data, setData] = useState<Bootstrap | null>(null);
  const [workspace, setWorkspace] = useState<Workspace | null>(null);
  const [section, setSection] = useState<AppSection>("workspace");
  const [activeServiceId, setActiveServiceId] = useState<string | null>(null);
  const [newProjectOpen, setNewProjectOpen] = useState(false);
  const [saveQuoteOpen, setSaveQuoteOpen] = useState(false);
  const [loading, setLoading] = useState(true);
  const [fatalError, setFatalError] = useState("");
  const [notice, setNotice] = useState("");
  const [undoService, setUndoService] = useState<QuoteService | null>(null);
  const [marketOverview, setMarketOverview] = useState<MarketOverview | null>(null);
  const [marketOverviewServiceId, setMarketOverviewServiceId] = useState<string | null>(null);
  const [marketJob, setMarketJob] = useState<MarketResearchJob | null>(null);
  const [settingsInitialTab, setSettingsInitialTab] = useState<SettingsTab>("general");
  const [settingsInitialCurrency, setSettingsInitialCurrency] = useState<Currency | undefined>();
  const [documentQuoteId, setDocumentQuoteId] = useState<string | null>(null);
  const [documentAfterSave, setDocumentAfterSave] = useState(false);
  const [calculationBusy, setCalculationBusy] = useState(false);
  const closeAllowed = useRef(false);
  const marketJobRef = useRef<string | null>(null);

  const onSaved = useCallback((saved: QuoteService) => {
    setWorkspace((current) => current ? { ...current, services: current.services.map((service) => service.id === saved.id ? saved : service) } : current);
  }, []);
  const autosave = useAutosave(onSaved);
  const flushAutosave = autosave.flushAll;

  useEffect(() => {
    let unlisten: (() => void) | undefined;
    void getCurrentWindow().onCloseRequested(async (event) => {
      if (closeAllowed.current) return;
      event.preventDefault();
      if (await flushAutosave()) {
        closeAllowed.current = true;
        await getCurrentWindow().close();
      } else {
        setNotice("No se pudo guardar todo. Reintentá antes de cerrar para no perder cambios.");
      }
    }).then((stop) => { unlisten = stop; });
    return () => unlisten?.();
  }, [flushAutosave]);

  const refresh = useCallback(async () => {
    const bootstrap = await api.bootstrap();
    setData(bootstrap);
    return bootstrap;
  }, []);

  useEffect(() => {
    let cancelled = false;
    void (async () => {
      try {
        const bootstrap = await api.bootstrap();
        if (cancelled) return;
        setData(bootstrap);
        document.documentElement.dataset.theme = bootstrap.settings.theme;
        if (bootstrap.settings.activeProjectId && bootstrap.projects.some((project) => project.id === bootstrap.settings.activeProjectId && project.status === "active")) {
          const current = await api.loadWorkspace(bootstrap.settings.activeProjectId);
          if (!cancelled) { setWorkspace(current); setActiveServiceId(current.services[0]?.id ?? null); }
        }
      } catch (error) {
        if (!cancelled) setFatalError(String(error));
      } finally { if (!cancelled) setLoading(false); }
    })();
    return () => { cancelled = true; };
  }, []);

  useEffect(() => {
    if (data) {
      document.documentElement.dataset.theme = data.settings.theme;
      document.documentElement.dataset.help = data.settings.helpMode;
    }
  }, [data]);

  useEffect(() => {
    let cancelled = false;
    if (!activeServiceId) return () => { cancelled = true; };
    void api.getMarketOverview(activeServiceId).then((overview) => { if (!cancelled) { setMarketOverview(overview); setMarketOverviewServiceId(activeServiceId); } }).catch(() => { if (!cancelled) { setMarketOverview({ latestSnapshot: null, observations: [], history: [] }); setMarketOverviewServiceId(activeServiceId); } });
    return () => { cancelled = true; };
  }, [activeServiceId]);

  const projectResult = useMemo(() => workspace && data ? evaluateWorkspace(workspace, data.settings, data.pricing) : null, [workspace, data]);

  async function createProject(input: CreateProjectInput) {
    const created = await api.createProject(input);
    setWorkspace(created); setActiveServiceId(null); setSection("workspace");
    await refresh();
  }

  async function openProject(id: string) {
    if (marketJob?.status === "RUNNING") { setNotice("Cancelá o esperá la actualización de mercado antes de cambiar de proyecto."); return; }
    if (!(await autosave.flushAll())) { setNotice("No se pudo guardar el proyecto actual. Reintentá antes de cambiar."); return; }
    try {
      const loaded = await api.loadWorkspace(id);
      setWorkspace(loaded); setActiveServiceId(loaded.services[0]?.id ?? null); setSection("workspace"); setNotice("");
      await refresh();
    } catch (error) { setNotice(String(error)); }
  }

  async function addService(type: ServiceType) {
    if (!workspace) return;
    const service = await api.addService(workspace.quote.id, type);
    setWorkspace({ ...workspace, services: [...workspace.services, service] });
    setActiveServiceId(service.id);
    await refresh();
  }

  function queueService(service: QuoteService, next: Partial<QuoteService>, immediate = false) {
    const updated = { ...service, ...next };
    setWorkspace((current) => current ? { ...current, services: current.services.map((item) => item.id === service.id ? updated : item) } : current);
    autosave.schedule({
      id: updated.id,
      title: updated.title,
      configurationVersion: updated.configurationVersion,
      configurationJson: updated.configurationJson,
      calculatedSubtotalMinor: updated.calculatedSubtotalMinor,
      suggestedSubtotalMinor: updated.suggestedSubtotalMinor,
      finalSubtotalMinor: updated.finalSubtotalMinor,
      hasOverride: updated.hasOverride,
      manualSubtotalMinor: updated.manualSubtotalMinor,
      manualReason: updated.manualReason,
      pricingSnapshotJson: updated.pricingSnapshotJson,
      serviceDefinitionVersion: updated.serviceDefinitionVersion,
      expectedRevision: service.rowRevision,
    }, immediate);
  }

  function videoChange(service: QuoteService, config: VideoConfiguration, manualMinor: number | null = service.manualSubtotalMinor, manualReason: string | null = service.manualReason, immediate = false) {
    if (!workspace || !data) return;
    const engineInput = { serviceType: service.serviceType, currency: workspace.quote.currency, parameterValues: config as unknown as Record<string, unknown>, externalCosts: config.externalCosts, fixedUrgencyMinor: config.urgencyFeeMinor, finalOverrideMinor: manualMinor, hasOverride: manualMinor != null, settings: data.settings, pricing: data.pricing };
    const result = runPricingEngine(engineInput);
    const snapshot = createPricingSnapshot(engineInput, result);
    const definition = data.pricing.definitions.find((item) => item.serviceType === service.serviceType);
    const envelope: ServiceConfigurationEnvelope<VideoConfiguration> = { schemaVersion: 1, serviceType: "video-editing", data: config };
    queueService(service, { configurationJson: JSON.stringify(envelope), calculatedSubtotalMinor: result.calculatedSubtotalMinor, suggestedSubtotalMinor: result.suggestedSubtotalMinor, finalSubtotalMinor: result.finalSubtotalMinor, hasOverride: manualMinor != null, manualSubtotalMinor: manualMinor, manualReason, pricingSnapshotJson: snapshot ? JSON.stringify(snapshot) : null, serviceDefinitionVersion: definition?.version ?? null }, immediate);
  }

  function programmingChange(service: QuoteService, config: ProgrammingConfiguration) {
    if (!workspace || !data) return;
    const engineInput = { serviceType: service.serviceType, currency: workspace.quote.currency, parameterValues: config.parameterValues, externalCosts: config.externalCosts, finalOverrideMinor: service.finalSubtotalMinor, hasOverride: service.hasOverride, settings: data.settings, pricing: data.pricing };
    const result = runPricingEngine(engineInput);
    const snapshot = createPricingSnapshot(engineInput, result);
    const definition = data.pricing.definitions.find((item) => item.serviceType === service.serviceType);
    const envelope: ServiceConfigurationEnvelope<ProgrammingConfiguration> = { schemaVersion: 2, serviceType: service.serviceType, data: config };
    queueService(service, { configurationVersion: 2, configurationJson: JSON.stringify(envelope), calculatedSubtotalMinor: result.calculatedSubtotalMinor, suggestedSubtotalMinor: result.suggestedSubtotalMinor, finalSubtotalMinor: result.finalSubtotalMinor, pricingSnapshotJson: snapshot ? JSON.stringify(snapshot) : null, serviceDefinitionVersion: definition?.version ?? null });
  }

  function genericEngineChange(service: QuoteService, config: ProductConfiguration | HybridConfiguration, immediate = false) {
    if (!workspace || !data) return;
    const engine = data.pricing.pricingEngines.find((item) => item.engineKey === service.serviceType);
    if (!engine) return;
    const profile = data.pricing.economicProfiles.find((item) => item.currency === workspace.quote.currency) ?? null;
    const context = { currency: workspace.quote.currency, hourlyRateMinor: activeHourlyRate(profile), usdToArsMicros: data.settings.usdToArsMicros };
    let result = engine.calculatorKey === "hybrid-v1" ? calculateHybrid(config as HybridConfiguration, context) : calculateProduct(config, context);
    if (service.hasOverride && service.manualSubtotalMinor != null) result = { ...result, finalSubtotalMinor: service.manualSubtotalMinor, effectiveSubtotalMinor: service.manualSubtotalMinor, hasOverride: true, lines: [...result.lines, { label: "Precio final manual", kind: "override", amountMinor: service.manualSubtotalMinor - (result.suggestedSubtotalMinor ?? result.calculatedSubtotalMinor ?? 0) }] };
    const envelope: ServiceConfigurationEnvelope<ProductConfiguration | HybridConfiguration> = { schemaVersion: 1, serviceType: service.serviceType, data: config };
    queueService(service, { configurationJson: JSON.stringify(envelope), calculatedSubtotalMinor: result.calculatedSubtotalMinor, suggestedSubtotalMinor: result.suggestedSubtotalMinor, finalSubtotalMinor: result.finalSubtotalMinor, pricingSnapshotJson: JSON.stringify({ schemaVersion: 1, createdAt: new Date().toISOString(), engineId: engine.id, result }), serviceDefinitionVersion: engine.classificationVersion }, immediate);
  }

  function finalPriceChange(service: QuoteService, finalMinor: number | null, reason: string | null) {
    if (service.serviceType === "video-editing") {
      const config = (JSON.parse(service.configurationJson) as ServiceConfigurationEnvelope<VideoConfiguration>).data;
      videoChange(service, config, finalMinor, reason, true);
      return;
    }
    const engine = data?.pricing.pricingEngines.find((item) => item.engineKey === service.serviceType);
    if (engine && (engine.calculatorKey === "physical-product-v1" || engine.calculatorKey === "hybrid-v1")) {
      const config = (JSON.parse(service.configurationJson) as ServiceConfigurationEnvelope<ProductConfiguration | HybridConfiguration>).data;
      genericEngineChange({ ...service, finalSubtotalMinor: finalMinor, manualSubtotalMinor: finalMinor, manualReason: reason, hasOverride: finalMinor != null }, config, true);
      return;
    }
    const config = (JSON.parse(service.configurationJson) as ServiceConfigurationEnvelope<ProgrammingConfiguration>).data;
    programmingChange({ ...service, finalSubtotalMinor: finalMinor, manualSubtotalMinor: finalMinor, manualReason: reason, hasOverride: finalMinor != null }, config);
  }

  async function deleteService(service: QuoteService) {
    if (!window.confirm(`¿Quitar “${service.title}” de la cotización?`)) return;
    if (!(await autosave.flushAll())) { setNotice("No se pudo guardar el servicio antes de quitarlo."); return; }
    await api.setServiceDeleted(service.id, true);
    setWorkspace((current) => current ? { ...current, services: current.services.filter((item) => item.id !== service.id) } : current);
    setActiveServiceId((current) => current === service.id ? null : current);
    setUndoService(service);
    setNotice(`“${service.title}” se quitó de la cotización.`);
    await refresh();
  }

  async function restoreDeletedService() {
    if (!undoService || !workspace) return;
    await api.setServiceDeleted(undoService.id, false);
    const reloaded = await api.loadWorkspace(workspace.project.id);
    setWorkspace(reloaded);
    setActiveServiceId(undoService.id);
    setUndoService(null);
    setNotice("Servicio recuperado.");
    await refresh();
  }

  async function moveService(service: QuoteService, direction: -1 | 1) {
    if (!workspace || !(await autosave.flushAll())) return;
    const currentIndex = workspace.services.findIndex((item) => item.id === service.id);
    const target = currentIndex + direction;
    if (target < 0 || target >= workspace.services.length) return;
    const ordered = [...workspace.services];
    [ordered[currentIndex], ordered[target]] = [ordered[target], ordered[currentIndex]];
    await api.reorderServices(workspace.quote.id, ordered.map((item) => item.id));
    const reloaded = await api.loadWorkspace(workspace.project.id);
    setWorkspace(reloaded);
  }

  async function savePreset(name: string, config: VideoConfiguration) {
    await api.savePreset({ serviceType: "video-editing", name, configurationVersion: 1, configurationJson: presetConfiguration(config) });
    await refresh(); setNotice("Preset guardado.");
  }

  async function updatePreset(preset: Preset, config: VideoConfiguration) {
    await api.savePreset({ id: preset.id, serviceType: preset.serviceType, name: preset.name, configurationVersion: 1, configurationJson: presetConfiguration(config) });
    await refresh(); setNotice("Preset actualizado.");
  }

  async function deletePreset(preset: Preset) { await api.deleteUserPreset(preset.id); await refresh(); setNotice("Preset eliminado."); }
  async function restorePreset(preset: Preset) { await api.restoreSystemPreset(preset.id); await refresh(); setNotice("Preset del sistema restaurado."); }

  async function saveClient(input: ClientInput) { await api.saveClient(input); await refresh(); }
  async function archiveClient(client: Client, archived: boolean) { await api.setClientArchived(client.id, archived); await refresh(); }
  async function archiveProject(project: ProjectSummary, archived: boolean) { await api.setProjectArchived(project.id, archived); if (archived && workspace?.project.id === project.id) { setWorkspace(null); setActiveServiceId(null); } await refresh(); }

  async function saveSettings(input: SettingsInput) {
    const settings = await api.updateSettings(input);
    const pricing = await api.loadPricing();
    setData((current) => current ? { ...current, settings, pricing } : current);
  }

  function pricingChange(pricing: PricingConfiguration) { setData((current) => current ? { ...current, pricing } : current); }

  function openEconomyForQuote() {
    if (!workspace) return;
    setSettingsInitialTab("economy");
    setSettingsInitialCurrency(workspace.quote.currency);
    setSection("settings");
    setNotice("");
  }

  function recalculationInput(service: QuoteService, currentWorkspace: Workspace): SaveServiceInput {
    if (!data) throw new Error("La configuración de precios no está disponible.");
    const engine = data.pricing.pricingEngines.find((item) => item.engineKey === service.serviceType);
    if (engine && ["physical-product-v1", "hybrid-v1"].includes(engine.calculatorKey)) {
      const config = (JSON.parse(service.configurationJson) as ServiceConfigurationEnvelope<ProductConfiguration | HybridConfiguration>).data;
      const profile = data.pricing.economicProfiles.find((item) => item.currency === currentWorkspace.quote.currency) ?? null;
      const context = { currency: currentWorkspace.quote.currency, hourlyRateMinor: activeHourlyRate(profile), usdToArsMicros: data.settings.usdToArsMicros };
      let result = engine.calculatorKey === "hybrid-v1" ? calculateHybrid(config as HybridConfiguration, context) : calculateProduct(config, context);
      if (service.hasOverride && service.manualSubtotalMinor != null) {
        result = { ...result, finalSubtotalMinor: service.manualSubtotalMinor, effectiveSubtotalMinor: service.manualSubtotalMinor, hasOverride: true, lines: [...result.lines, { label: "Precio final manual", kind: "override", amountMinor: service.manualSubtotalMinor - (result.suggestedSubtotalMinor ?? result.calculatedSubtotalMinor ?? 0) }] };
      }
      return {
        id: service.id, title: service.title, configurationVersion: service.configurationVersion,
        configurationJson: service.configurationJson, calculatedSubtotalMinor: result.calculatedSubtotalMinor,
        suggestedSubtotalMinor: result.suggestedSubtotalMinor, finalSubtotalMinor: result.finalSubtotalMinor,
        hasOverride: service.hasOverride, manualSubtotalMinor: service.manualSubtotalMinor, manualReason: service.manualReason,
        pricingSnapshotJson: JSON.stringify({ schemaVersion: 1, createdAt: new Date().toISOString(), engineId: engine.id, result }),
        serviceDefinitionVersion: engine.classificationVersion, expectedRevision: service.rowRevision,
      };
    }

    const config = service.serviceType === "video-editing"
      ? parseVideoEnvelope(service.configurationJson).data
      : parseProgrammingEnvelope(service.configurationJson).data;
    const parameterValues = service.serviceType === "video-editing"
      ? config as unknown as Record<string, unknown>
      : (config as ProgrammingConfiguration).parameterValues;
    const externalCosts = (config as VideoConfiguration | ProgrammingConfiguration).externalCosts;
    const engineInput = {
      serviceType: service.serviceType, currency: currentWorkspace.quote.currency, parameterValues, externalCosts,
      fixedUrgencyMinor: service.serviceType === "video-editing" ? (config as VideoConfiguration).urgencyFeeMinor : undefined,
      finalOverrideMinor: service.manualSubtotalMinor, hasOverride: service.hasOverride,
      settings: data.settings, pricing: data.pricing,
    };
    const result = runPricingEngine(engineInput);
    const snapshot = createPricingSnapshot(engineInput, result);
    const definition = data.pricing.definitions.find((item) => item.serviceType === service.serviceType);
    return {
      id: service.id, title: service.title, configurationVersion: service.configurationVersion,
      configurationJson: service.configurationJson, calculatedSubtotalMinor: result.calculatedSubtotalMinor,
      suggestedSubtotalMinor: result.suggestedSubtotalMinor, finalSubtotalMinor: result.finalSubtotalMinor,
      hasOverride: service.hasOverride, manualSubtotalMinor: service.manualSubtotalMinor, manualReason: service.manualReason,
      pricingSnapshotJson: snapshot ? JSON.stringify(snapshot) : null,
      serviceDefinitionVersion: definition?.version ?? null, expectedRevision: service.rowRevision,
    };
  }

  async function calculateEstimate(automatic = true) {
    if (!workspace || !data || calculationBusy) return;
    setCalculationBusy(true);
    try {
      if (!(await autosave.flushAll())) {
        setNotice("No se pudieron guardar los cambios antes de calcular. Corregí el error indicado y reintentá.");
        return;
      }
      let reloaded = await api.loadWorkspace(workspace.project.id);
      for (const service of reloaded.services) await api.saveService(recalculationInput(service, reloaded));
      reloaded = await api.loadWorkspace(workspace.project.id);
      setWorkspace(reloaded);
      const evaluated = evaluateWorkspace(reloaded, data.settings, data.pricing);
      if (evaluated.totalMinor != null && !evaluated.isPartial) {
        if (automatic && activeServiceId) {
          setNotice("Cálculo interno listo. Actualizando las fuentes confiables disponibles…");
          await startMarketResearchFor(activeServiceId, false, reloaded.project.id);
        } else {
          setNotice("Estimado calculado con tus datos manuales. Revisá el total y el desglose.");
        }
        return;
      }
      const activeIssues = evaluated.services.find(({ service }) => service.id === activeServiceId)?.result.issues ?? [];
      const economyMissing = activeIssues.some((issue) => /configurá tu (economía|tarifa)/i.test(issue));
      setNotice(economyMissing
        ? `Para calcular el estimado falta configurar tu tarifa en ${reloaded.quote.currency}.`
        : activeIssues.length > 0
          ? `Para calcular el estimado falta: ${activeIssues.join(" ")}`
          : "Completá los datos pendientes de cada módulo para calcular el estimado.");
    } catch (error) {
      setNotice(`No se pudo actualizar el estimado: ${String(error)}`);
    } finally {
      setCalculationBusy(false);
    }
  }

  async function toggleTheme() {
    if (!data) return;
    await saveSettings({ theme: data.settings.theme === "warm" ? "dark" : "warm", hourlyRateArsMinor: data.settings.hourlyRateArsMinor, hourlyRateUsdMinor: data.settings.hourlyRateUsdMinor, usdToArsMicros: data.settings.usdToArsMicros, suggestionsEnabled: data.settings.suggestionsEnabled, suggestionStrategy: data.settings.suggestionStrategy, baseCurrency: data.settings.baseCurrency, helpMode: data.settings.helpMode, localAiEnabled: data.settings.localAiEnabled, ollamaBaseUrl: data.settings.ollamaBaseUrl, ollamaModel: data.settings.ollamaModel, aiAutoApplyHighConfidence: data.settings.aiAutoApplyHighConfidence });
  }

  async function changeCurrency(currency: Currency) {
    if (!workspace || currency === workspace.quote.currency) return;
    if (!(await autosave.flushAll())) return setNotice("Guardá los cambios pendientes antes de cambiar la moneda.");
    try { const changed = await api.changeCurrency(workspace.project.id, currency); setWorkspace(changed); await refresh(); }
    catch (error) { setNotice(String(error)); }
  }

  async function updateMarket(force = false) {
    if (!activeServiceId || !workspace || marketJob?.status === "RUNNING") return;
    if (!(await autosave.flushAll())) { setNotice("No se pudo guardar el servicio antes de investigar el mercado."); return; }
    await startMarketResearchFor(activeServiceId, force, workspace.project.id);
  }

  async function startMarketResearchFor(serviceId: string, force: boolean, projectId: string) {
    try {
      const started = await api.startMarketResearch(serviceId, force);
      marketJobRef.current = started.id; setMarketJob(started); setNotice("");
      void pollMarket(started.id, serviceId, projectId);
    } catch (error) { setNotice(String(error)); }
  }

  async function pollMarket(jobId: string, serviceId: string, projectId: string) {
    while (marketJobRef.current === jobId) {
      await new Promise((resolve) => window.setTimeout(resolve, 500));
      try {
        const next = await api.getMarketResearchJob(jobId);
        if (marketJobRef.current !== jobId) return;
        setMarketJob(next);
        if (next.status !== "RUNNING") {
          marketJobRef.current = null;
          if (next.status === "COMPLETED") {
            const overview = await api.getMarketOverview(serviceId);
            const success = next.items.filter((item) => item.status === "SUCCESS").length;
            const cached = next.items.filter((item) => item.status === "CACHED").length;
            const manual = next.items.filter((item) => item.status === "MANUAL").length;
            const unavailable = next.items.filter((item) => ["ERROR", "BLOCKED", "NEEDS_CONFIGURATION"].includes(item.status)).length;
            setMarketOverview(overview);
            setMarketOverviewServiceId(serviceId);
            if (next.suggestionUpdateStatus === "APPLIED") {
              const latestWorkspace = await api.loadWorkspace(projectId);
              setWorkspace(latestWorkspace);
            }
            setNotice(next.error || next.suggestionUpdateMessage || `${success} actualizadas · ${cached} en caché · ${manual} manuales · ${unavailable} no disponibles. La evidencia quedó separada del precio final.`);
            await refresh();
          } else if (next.status === "ERROR") setNotice(next.error || "La investigación no pudo completarse.");
          else setNotice("Actualización de mercado cancelada.");
          return;
        }
      } catch (error) { marketJobRef.current = null; setNotice(String(error)); return; }
    }
  }

  async function cancelMarket() {
    if (!marketJob || marketJob.status !== "RUNNING") return;
    const cancelled = await api.cancelMarketResearch(marketJob.id); setMarketJob(cancelled);
  }

  async function generateClientDocument() {
    if (!workspace || !projectResult) return;
    if (projectResult.totalMinor == null || projectResult.isPartial) {
      setNotice("Completá todos los módulos y calculá un total antes de generar un presupuesto para cliente.");
      return;
    }
    if (!(await autosave.flushAll())) {
      setNotice("No se pudieron guardar los cambios antes de preparar el presupuesto.");
      return;
    }
    setDocumentAfterSave(true);
    setSaveQuoteOpen(true);
  }

  async function openSaveQuote() {
    if (!workspace) return;
    if (!(await autosave.flushAll())) { setNotice("No se pudo completar el autosave. Reintentá antes de crear el snapshot histórico."); return; }
    setSaveQuoteOpen(true);
  }

  async function saveQuoteSnapshot(input: import("../domain/types").SaveQuoteSnapshotInput) {
    await api.saveQuoteSnapshot(input);
    if (workspace) {
      const reloaded = await api.loadWorkspace(workspace.project.id);
      setWorkspace(reloaded);
    }
    await refresh();
    if (documentAfterSave) {
      setDocumentAfterSave(false);
      setDocumentQuoteId(input.quoteId);
      setNotice("Cotización guardada. Completá los datos públicos y exportá el PDF.");
      return;
    }
    setNotice(input.reason === "calculation_update" ? "Nueva revisión histórica guardada. Las revisiones anteriores siguen intactas." : "Cotización guardada en el historial.");
  }

  function useDuplicatedQuote(duplicated: Workspace) {
    setWorkspace(duplicated);
    setActiveServiceId(duplicated.services[0]?.id ?? null);
    setSection("workspace");
    setNotice("Proyecto nuevo creado desde el snapshot. Conserva los precios guardados y no fue recalculado.");
    void refresh();
  }

  if (loading) return <div className="startup"><div className="brand__mark"><i /><i /></div><span>Preparando tu espacio de trabajo…</span></div>;
  if (fatalError || !data) return <div className="startup startup--error"><strong>No se pudo abrir Pricing OS</strong><p>{fatalError}</p><Button onClick={() => window.location.reload()}>Reintentar</Button></div>;

  const activeProject = workspace?.project ?? null;
  let content: React.ReactNode;
  if (section === "workspace") content = workspace && projectResult ? <WorkspaceView workspace={workspace} settings={data.settings} pricing={data.pricing} result={projectResult} presets={data.presets} statuses={autosave.statuses} errors={autosave.errors} activeServiceId={activeServiceId} onActiveService={setActiveServiceId} onAddService={addService} onVideoChange={videoChange} onProgrammingChange={programmingChange} onGenericEngineChange={genericEngineChange} onFinalPriceChange={finalPriceChange} onTitleChange={(service, title) => queueService(service, { title })} onDeleteService={deleteService} onMoveService={moveService} onRetry={autosave.retry} onSavePreset={savePreset} onUpdatePreset={updatePreset} onDeletePreset={deletePreset} onRestorePreset={restorePreset} market={marketOverviewServiceId === activeServiceId ? marketOverview : null} marketJob={marketJob?.quoteServiceId === activeServiceId ? marketJob : null} onUpdateMarket={updateMarket} onCancelMarket={cancelMarket} onSaveQuote={openSaveQuote} onCalculateEstimate={calculateEstimate} onConfigureEconomy={openEconomyForQuote} calculationBusy={calculationBusy} onGenerateDocument={generateClientDocument} documentBusy={documentAfterSave} marketUpdating={marketJob?.status === "RUNNING"} /> : <div className="view-page"><EmptyState eyebrow="Pricing OS" title="Creá tu primer proyecto" description="El proyecto organiza cliente, moneda, cotización y servicios en un único workspace." action={<Button variant="accent" onClick={() => setNewProjectOpen(true)}>Nuevo proyecto</Button>} /></div>;
  else if (section === "clients") content = <ClientsView clients={data.clients} projects={data.projects} onSave={saveClient} onArchive={archiveClient} onOpenProject={openProject} />;
  else if (section === "projects") content = <ProjectsView projects={data.projects} onNew={() => setNewProjectOpen(true)} onOpen={openProject} onArchive={archiveProject} />;
  else if (section === "market") content = <MarketView pricing={data.pricing} activeServiceId={activeServiceId} job={marketJob} onResearch={updateMarket} onCancel={cancelMarket} onConfigureSources={() => { setSettingsInitialTab("sources"); setSettingsInitialCurrency(undefined); setSection("settings"); }} />;
  else if (section === "settings" || section === "services") content = <SettingsView key={`${section}-${settingsInitialTab}-${settingsInitialCurrency ?? ""}`} settings={data.settings} pricing={data.pricing} initialTab={section === "services" ? "engines" : settingsInitialTab} initialEconomyCurrency={settingsInitialCurrency} onSave={saveSettings} onPricingChange={pricingChange} />;
  else content = <QuotesHistoryView clients={data.clients} pricing={data.pricing} onOpenProject={openProject} onDuplicated={useDuplicatedQuote} />;

  return <div className="app-shell">
    <Sidebar section={section} onSection={(next) => { if (next === "settings") { setSettingsInitialTab("general"); setSettingsInitialCurrency(undefined); } setSection(next); }} onNewProject={() => setNewProjectOpen(true)} />
    <div className="app-body"><Topbar project={activeProject} projects={data.projects} theme={data.settings.theme} usdToArsMicros={data.settings.usdToArsMicros} onProject={openProject} onNewProject={() => setNewProjectOpen(true)} onCurrency={changeCurrency} onToggleTheme={toggleTheme} onSettings={() => { setSettingsInitialTab("general"); setSettingsInitialCurrency(undefined); setSection("settings"); }} />{notice && <div className="notice" role="status"><span>{notice}</span>{undoService && <button onClick={restoreDeletedService}>Deshacer</button>}<button onClick={() => { setNotice(""); setUndoService(null); }}>Cerrar</button></div>}{content}</div>
    {newProjectOpen && <NewProjectModal clients={data.clients} onClose={() => setNewProjectOpen(false)} onCreate={createProject} />}
    {saveQuoteOpen && workspace && projectResult && <SaveQuoteModal workspace={workspace} result={projectResult} onClose={() => { setSaveQuoteOpen(false); setDocumentAfterSave(false); }} onSave={saveQuoteSnapshot} title={documentAfterSave ? "Guardar y preparar presupuesto" : undefined} submitLabel={documentAfterSave ? "Guardar y continuar" : undefined} />}
    {documentQuoteId && workspace && <ClientDocumentModal quoteId={documentQuoteId} services={workspace.services.map((service) => ({ id: service.id, title: service.title }))} onClose={() => setDocumentQuoteId(null)} />}
  </div>;
}
