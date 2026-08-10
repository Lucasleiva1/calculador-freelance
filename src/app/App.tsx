import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { Clock3 } from "lucide-react";
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
} from "../domain/types";
import type { VideoConfiguration } from "../domain/video";
import type { ProgrammingConfiguration } from "../domain/programming";
import { evaluateWorkspace } from "../domain/quote";
import { createPricingSnapshot, runPricingEngine } from "../domain/pricingEngine";
import { api } from "../services/api";
import { useAutosave } from "../hooks/useAutosave";
import { Sidebar, type AppSection } from "../components/Sidebar";
import { Topbar } from "../components/Topbar";
import { NewProjectModal } from "../components/NewProjectModal";
import { EmptyState, Button } from "../components/ui";
import { WorkspaceView } from "../features/quotes/WorkspaceView";
import { ClientsView } from "../features/clients/ClientsView";
import { ProjectsView } from "../features/projects/ProjectsView";
import { SettingsView } from "../features/settings/SettingsView";
import { FutureView } from "../components/FutureView";

function presetConfiguration(config: VideoConfiguration) {
  return JSON.stringify(Object.fromEntries(Object.entries(config).filter(([key]) => !["estimatedHours", "externalCosts", "urgencyFeeMinor"].includes(key))));
}

export function App() {
  const [data, setData] = useState<Bootstrap | null>(null);
  const [workspace, setWorkspace] = useState<Workspace | null>(null);
  const [section, setSection] = useState<AppSection>("workspace");
  const [activeServiceId, setActiveServiceId] = useState<string | null>(null);
  const [newProjectOpen, setNewProjectOpen] = useState(false);
  const [loading, setLoading] = useState(true);
  const [fatalError, setFatalError] = useState("");
  const [notice, setNotice] = useState("");
  const [undoService, setUndoService] = useState<QuoteService | null>(null);
  const closeAllowed = useRef(false);

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
        await getCurrentWindow().destroy();
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
    if (data) document.documentElement.dataset.theme = data.settings.theme;
  }, [data]);

  const projectResult = useMemo(() => workspace && data ? evaluateWorkspace(workspace, data.settings, data.pricing) : null, [workspace, data]);

  async function createProject(input: CreateProjectInput) {
    const created = await api.createProject(input);
    setWorkspace(created); setActiveServiceId(null); setSection("workspace");
    await refresh();
  }

  async function openProject(id: string) {
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
    const envelope: ServiceConfigurationEnvelope<ProgrammingConfiguration> = { schemaVersion: 2, serviceType: "programming", data: config };
    queueService(service, { configurationVersion: 2, configurationJson: JSON.stringify(envelope), calculatedSubtotalMinor: result.calculatedSubtotalMinor, suggestedSubtotalMinor: result.suggestedSubtotalMinor, finalSubtotalMinor: result.finalSubtotalMinor, pricingSnapshotJson: snapshot ? JSON.stringify(snapshot) : null, serviceDefinitionVersion: definition?.version ?? null });
  }

  function finalPriceChange(service: QuoteService, finalMinor: number | null, reason: string | null) {
    if (service.serviceType === "video-editing") {
      const config = (JSON.parse(service.configurationJson) as ServiceConfigurationEnvelope<VideoConfiguration>).data;
      videoChange(service, config, finalMinor, reason, true);
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

  async function toggleTheme() {
    if (!data) return;
    await saveSettings({ theme: data.settings.theme === "warm" ? "dark" : "warm", hourlyRateArsMinor: data.settings.hourlyRateArsMinor, hourlyRateUsdMinor: data.settings.hourlyRateUsdMinor, usdToArsMicros: data.settings.usdToArsMicros, suggestionsEnabled: data.settings.suggestionsEnabled, suggestionStrategy: data.settings.suggestionStrategy, baseCurrency: data.settings.baseCurrency });
  }

  async function changeCurrency(currency: Currency) {
    if (!workspace || currency === workspace.quote.currency) return;
    if (!(await autosave.flushAll())) return setNotice("Guardá los cambios pendientes antes de cambiar la moneda.");
    try { const changed = await api.changeCurrency(workspace.project.id, currency); setWorkspace(changed); await refresh(); }
    catch (error) { setNotice(String(error)); }
  }

  if (loading) return <div className="startup"><div className="brand__mark"><i /><i /></div><span>Preparando tu espacio de trabajo…</span></div>;
  if (fatalError || !data) return <div className="startup startup--error"><strong>No se pudo abrir Pricing OS</strong><p>{fatalError}</p><Button onClick={() => window.location.reload()}>Reintentar</Button></div>;

  const activeProject = workspace?.project ?? null;
  let content: React.ReactNode;
  if (section === "workspace") content = workspace && projectResult ? <WorkspaceView workspace={workspace} settings={data.settings} pricing={data.pricing} result={projectResult} presets={data.presets} statuses={autosave.statuses} errors={autosave.errors} activeServiceId={activeServiceId} onActiveService={setActiveServiceId} onAddService={addService} onVideoChange={videoChange} onProgrammingChange={programmingChange} onFinalPriceChange={finalPriceChange} onTitleChange={(service, title) => queueService(service, { title })} onDeleteService={deleteService} onMoveService={moveService} onRetry={autosave.retry} onSavePreset={savePreset} onUpdatePreset={updatePreset} onDeletePreset={deletePreset} onRestorePreset={restorePreset} /> : <div className="view-page"><EmptyState eyebrow="Pricing OS" title="Creá tu primer proyecto" description="El proyecto organiza cliente, moneda, cotización y servicios en un único workspace." action={<Button variant="accent" onClick={() => setNewProjectOpen(true)}>Nuevo proyecto</Button>} /></div>;
  else if (section === "clients") content = <ClientsView clients={data.clients} projects={data.projects} onSave={saveClient} onArchive={archiveClient} onOpenProject={openProject} />;
  else if (section === "projects") content = <ProjectsView projects={data.projects} onNew={() => setNewProjectOpen(true)} onOpen={openProject} onArchive={archiveProject} />;
  else if (section === "settings" || section === "services" || section === "market") content = <SettingsView key={section} settings={data.settings} pricing={data.pricing} initialTab={section === "services" ? "services" : section === "market" ? "sources" : "general"} onSave={saveSettings} onPricingChange={pricingChange} />;
  else content = <FutureView eyebrow="Versionado futuro" title="Historial" description="Las cotizaciones ya tienen versión y estado; el historial comparativo se activará más adelante." icon={Clock3} />;

  return <div className="app-shell">
    <Sidebar section={section} onSection={setSection} onNewProject={() => setNewProjectOpen(true)} />
    <div className="app-body"><Topbar project={activeProject} projects={data.projects} theme={data.settings.theme} usdToArsMicros={data.settings.usdToArsMicros} onProject={openProject} onNewProject={() => setNewProjectOpen(true)} onCurrency={changeCurrency} onToggleTheme={toggleTheme} onSettings={() => setSection("settings")} />{notice && <div className="notice" role="status"><span>{notice}</span>{undoService && <button onClick={restoreDeletedService}>Deshacer</button>}<button onClick={() => { setNotice(""); setUndoService(null); }}>Cerrar</button></div>}{content}</div>
    {newProjectOpen && <NewProjectModal clients={data.clients} onClose={() => setNewProjectOpen(false)} onCreate={createProject} />}
  </div>;
}
