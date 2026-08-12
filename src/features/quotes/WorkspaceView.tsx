import { useEffect, useMemo, useState } from "react";
import { ArrowDown, ArrowUp, Blend, Calculator, CircleAlert, CircleCheck, Code2, Film, FileOutput, FileText, Package, Plus, Save, Settings2, Shirt, Trash2 } from "lucide-react";
import type {
  AppSettings,
  MarketOverview,
  MarketResearchJob,
  PricingConfiguration,
  Preset,
  QuoteService,
  SaveStatus,
  ServiceConfigurationEnvelope,
  ServiceType,
  Workspace,
} from "../../domain/types";
import type { VideoConfiguration } from "../../domain/video";
import type { ProgrammingConfiguration } from "../../domain/programming";
import type { HybridConfiguration, ProductConfiguration } from "../../domain/product";
import { parseProgrammingEnvelope } from "../../domain/programming";
import { parseProfessionalEnvelope } from "../../domain/professional";
import { DynamicFields } from "../../components/DynamicFields";
import type { ProjectResult } from "../../domain/quote";
import { formatMoney } from "../../domain/money";
import { VideoEditor } from "../video/VideoEditor";
import { ProgrammingEditor } from "../programming/ProgrammingEditor";
import { PrintDesignEditor } from "../print-design/PrintDesignEditor";
import { ProductEditor } from "../product/ProductEditor";
import { ResultInspector } from "../../components/ResultInspector";
import { Button, EmptyState, StatusDot } from "../../components/ui";

function saveLabel(status: SaveStatus | undefined) {
  if (status === "saving") return "Guardando";
  if (status === "error") return "Error al guardar";
  return "Guardado";
}

export function WorkspaceView({
  workspace,
  settings,
  pricing,
  result,
  presets,
  statuses,
  errors,
  activeServiceId,
  onActiveService,
  onAddService,
  onVideoChange,
  onProgrammingChange,
  onGenericEngineChange,
  onFinalPriceChange,
  onTitleChange,
  onDeleteService,
  onMoveService,
  onRetry,
  onSavePreset,
  onUpdatePreset,
  onDeletePreset,
  onRestorePreset,
  market,
  marketJob,
  onUpdateMarket,
  onCancelMarket,
  onSaveQuote,
  onCalculateEstimate,
  onConfigureEconomy,
  calculationBusy = false,
  onGenerateDocument,
  documentReady = false,
  documentBusy = false,
  marketUpdating,
}: {
  workspace: Workspace;
  settings: AppSettings;
  pricing: PricingConfiguration;
  result: ProjectResult;
  presets: Preset[];
  statuses: Record<string, SaveStatus>;
  errors: Record<string, string>;
  activeServiceId: string | null;
  onActiveService: (id: string) => void;
  onAddService: (type: ServiceType) => Promise<void>;
  onVideoChange: (service: QuoteService, config: VideoConfiguration, manualMinor?: number | null, manualReason?: string | null, immediate?: boolean) => void;
  onProgrammingChange: (service: QuoteService, config: ProgrammingConfiguration) => void;
  onGenericEngineChange: (service: QuoteService, config: ProductConfiguration | HybridConfiguration, immediate?: boolean) => void;
  onFinalPriceChange: (service: QuoteService, finalMinor: number | null, reason: string | null) => void;
  onTitleChange: (service: QuoteService, title: string) => void;
  onDeleteService: (service: QuoteService) => Promise<void>;
  onMoveService: (service: QuoteService, direction: -1 | 1) => Promise<void>;
  onRetry: (id: string) => void;
  onSavePreset: (name: string, config: VideoConfiguration) => Promise<void>;
  onUpdatePreset: (preset: Preset, config: VideoConfiguration) => Promise<void>;
  onDeletePreset: (preset: Preset) => Promise<void>;
  onRestorePreset: (preset: Preset) => Promise<void>;
  market: MarketOverview | null;
  marketJob: MarketResearchJob | null;
  onUpdateMarket: (force?: boolean) => Promise<void>;
  onCancelMarket: () => Promise<void>;
  onSaveQuote: () => Promise<void>;
  /** Recalcula y persiste el estimado actual antes de mostrarlo. */
  onCalculateEstimate?: () => Promise<void> | void;
  /** Lleva a la configuración de economía/tarifa requerida por el cálculo. */
  onConfigureEconomy?: () => void;
  calculationBusy?: boolean;
  /** Abre el flujo real que permite preparar, previsualizar y exportar el presupuesto. */
  onGenerateDocument?: () => Promise<void> | void;
  documentReady?: boolean;
  documentBusy?: boolean;
  /** Bloquea únicamente la edición del módulo mientras se investiga el mercado. */
  marketUpdating?: boolean;
}) {
  const [addOpen, setAddOpen] = useState(false);
  const [localCalculationBusy, setLocalCalculationBusy] = useState(false);
  useEffect(() => {
    if (!activeServiceId && workspace.services[0]) onActiveService(workspace.services[0].id);
  }, [activeServiceId, onActiveService, workspace.services]);
  const active = workspace.services.find((service) => service.id === activeServiceId) ?? null;
  const activeStatus = active ? statuses[active.id] : undefined;
  const activeResult = result.services.find(({ service }) => service.id === active?.id)?.result;
  const activeEngine = pricing.pricingEngines.find((engine) => engine.engineKey === active?.serviceType) ?? null;
  const videoConfig = useMemo(() => {
    if (!active || active.serviceType !== "video-editing") return null;
    try { return (JSON.parse(active.configurationJson) as ServiceConfigurationEnvelope<VideoConfiguration>).data; }
    catch { return null; }
  }, [active]);
  const programmingConfig = useMemo(() => {
    if (!active || active.serviceType === "video-editing" || activeEngine?.calculatorKey !== "professional-service-v1") return null;
    try { return active.serviceType === "programming" ? parseProgrammingEnvelope(active.configurationJson).data : parseProfessionalEnvelope(active.configurationJson, active.serviceType).data; }
    catch { return { parameterValues: {}, externalCosts: [], notes: "" }; }
  }, [active, activeEngine?.calculatorKey]);
  const productConfig = useMemo(() => {
    if (!active || !activeEngine || !["physical-product-v1", "hybrid-v1"].includes(activeEngine.calculatorKey)) return null;
    try { return (JSON.parse(active.configurationJson) as ServiceConfigurationEnvelope<ProductConfiguration | HybridConfiguration>).data; }
    catch { return null; }
  }, [active, activeEngine]);
  const activeEngines = pricing.pricingEngines.filter((engine) => engine.status === "active" && engine.calculatorKey !== "unconfigured");
  const activeIssues = activeResult?.issues ?? [];
  const activeIsReady = activeResult?.status === "ready" && activeResult.finalSubtotalMinor != null;
  const projectReadyForDocument = workspace.services.length > 0 && result.totalMinor != null && result.unpricedCount === 0;
  const requiresEconomy = activeIssues.some((issue) => /Configurá tu (tarifa|economía)/u.test(issue));
  const isMarketUpdating = marketUpdating ?? marketJob?.status === "RUNNING";
  const calculating = calculationBusy || localCalculationBusy;

  function revealEstimate() {
    const target = document.getElementById("resultado-estimado");
    target?.scrollIntoView?.({ behavior: "smooth", block: "nearest" });
    target?.focus({ preventScroll: true });
  }

  async function calculateAndReveal() {
    if (!active || isMarketUpdating || calculating) return;
    setLocalCalculationBusy(true);
    try {
      await onCalculateEstimate?.();
    } finally {
      setLocalCalculationBusy(false);
      window.setTimeout(revealEstimate, 0);
    }
  }

  function EngineIcon({ type, size = 16 }: { type: string; size?: number }) {
    if (type === "video-editing") return <Film size={size} />;
    if (type === "print-design") return <Shirt size={size} />;
    const engine = pricing.pricingEngines.find((item) => item.engineKey === type);
    if (engine?.engineType === "product") return <Package size={size} />;
    if (engine?.engineType === "hybrid") return <Blend size={size} />;
    return <Code2 size={size} />;
  }

  return <div className="workspace-layout">
    <main className="workspace-main">
      <div className="workspace-scroll">
        <header className="page-header"><div><span className="eyebrow">Cotización · Draft v{workspace.quote.version}{workspace.quote.snapshotRevision > 0 ? ` · Historial rev. ${workspace.quote.snapshotRevision}` : ""}</span><h1>Cotización</h1><p>Construí el proyecto por módulos independientes. El borrador se guarda automáticamente.</p></div><div className={`save-indicator save-indicator--${activeStatus ?? "saved"}`}><StatusDot tone={activeStatus === "error" ? "danger" : activeStatus === "saving" ? "muted" : "accent"} />{saveLabel(activeStatus)}{activeStatus === "error" && active && <button onClick={() => onRetry(active.id)}>Reintentar</button>}</div></header>
        <div className="service-tabs">
          {workspace.services.map((service, index) => <button key={service.id} className={active?.id === service.id ? "is-active" : ""} onClick={() => onActiveService(service.id)}><span>{String(index + 1).padStart(2, "0")}</span><EngineIcon type={service.serviceType} /><strong>{service.title}</strong></button>)}
          <div className="add-service"><button className="add-service__trigger" onClick={() => setAddOpen(!addOpen)}><Plus size={17} /> Agregar módulo</button>{addOpen && <div className="add-service__menu">{activeEngines.map((engine) => <button key={engine.id} onClick={async () => { setAddOpen(false); await onAddService(engine.engineKey); }}><EngineIcon type={engine.engineKey} size={17} /><span>{engine.name}<small>{engine.engineType === "product" ? "Producto físico" : engine.engineType === "hybrid" ? "Servicio + producto" : engine.engineKey === "video-editing" ? "Editor audiovisual" : "Servicio profesional"}</small></span></button>)}<button onClick={() => { setAddOpen(false); }} disabled><Plus size={17} /><span>Nuevo motor<small>Crealo desde Servicios</small></span></button></div>}</div>
        </div>

        {!active ? <EmptyState eyebrow="Cotización vacía" title="Agregá el primer módulo" description="Los servicios, productos e híbridos viven como motores independientes dentro del mismo proyecto." action={<Button variant="accent" onClick={() => onAddService("video-editing")}><Plus size={17} /> Agregar Edición de video</Button>} /> : <section className="service-panel">
          <header className="service-panel__header"><div className="service-title"><span>{String(workspace.services.indexOf(active) + 1).padStart(2, "0")} /</span><input aria-label="Título del módulo" value={active.title} disabled={isMarketUpdating} onChange={(event) => onTitleChange(active, event.target.value)} /></div><div className="service-panel__tools"><span>{formatMoney(activeResult?.effectiveSubtotalMinor, workspace.quote.currency)}</span><button title="Mover arriba" disabled={isMarketUpdating || workspace.services.indexOf(active) === 0} onClick={() => onMoveService(active, -1)}><ArrowUp size={16} /></button><button title="Mover abajo" disabled={isMarketUpdating || workspace.services.indexOf(active) === workspace.services.length - 1} onClick={() => onMoveService(active, 1)}><ArrowDown size={16} /></button><button title="Quitar módulo" disabled={isMarketUpdating} onClick={() => onDeleteService(active)}><Trash2 size={16} /></button></div></header>
          {activeStatus === "error" && <div className="save-error" role="alert">{errors[active.id]}</div>}
          <section className={`estimate-callout ${activeIsReady ? "estimate-callout--ready" : "estimate-callout--incomplete"}`} aria-live="polite">
            <div className="estimate-callout__status">{activeIsReady ? <CircleCheck size={20} aria-hidden="true" /> : <CircleAlert size={20} aria-hidden="true" />}<div><span className="eyebrow">{activeIsReady ? "Estimado actualizado" : "Estimado pendiente"}</span><strong>{activeIsReady ? formatMoney(activeResult?.finalSubtotalMinor ?? null, workspace.quote.currency) : "Completá los requisitos para ver el precio"}</strong><p>{activeIsReady ? "El cálculo se actualiza cuando cambiás los parámetros de este módulo." : "No se inventa ningún importe: revisá lo que falta y volvé a calcular."}</p></div></div>
            <div className="estimate-callout__actions"><Button type="button" variant={activeIsReady ? "default" : "accent"} onClick={() => void calculateAndReveal()} disabled={calculating || isMarketUpdating}><Calculator size={16} /> {calculating ? "Calculando…" : "Calcular los 3 precios"}</Button>{requiresEconomy && onConfigureEconomy && <Button type="button" variant="ghost" onClick={onConfigureEconomy}><Settings2 size={16} /> Configurar precio local</Button>}</div>
            {!activeIsReady && activeIssues.length > 0 && <ul className="estimate-callout__requirements">{activeIssues.slice(0, 3).map((issue) => <li key={issue}>{issue}</li>)}</ul>}
          </section>
          {isMarketUpdating && <p id="market-update-lock" className="market-update-lock" role="status"><CircleAlert size={17} aria-hidden="true" /> Actualizando la referencia de mercado. Tus parámetros quedan bloqueados para conservar exactamente los valores que cargaste.</p>}
          <fieldset className="service-editor" disabled={isMarketUpdating} aria-describedby={isMarketUpdating ? "market-update-lock" : undefined}><legend className="sr-only">Campos editables del módulo</legend>
            {active.serviceType === "video-editing" && videoConfig && <><VideoEditor service={active} config={videoConfig} currency={workspace.quote.currency} presets={presets.filter((preset) => preset.serviceType === "video-editing")} onChange={(config, manual, reason, immediate) => onVideoChange(active, config, manual, reason, immediate)} onSavePreset={onSavePreset} onUpdatePreset={onUpdatePreset} onDeletePreset={onDeletePreset} onRestorePreset={onRestorePreset} />{pricing.parameters.some((item) => item.serviceDefinitionId === pricing.definitions.find((definition) => definition.serviceType === "video-editing")?.id && !item.uiManaged && item.enabled) && <section className="editor-section"><span className="eyebrow">Parámetros personalizados</span><DynamicFields parameters={pricing.parameters.filter((item) => item.serviceDefinitionId === pricing.definitions.find((definition) => definition.serviceType === "video-editing")?.id && !item.uiManaged)} options={pricing.options} values={videoConfig as unknown as Record<string, unknown>} suggestionsEnabled={settings.suggestionsEnabled} onChange={(values) => onVideoChange(active, values as unknown as VideoConfiguration)} /></section>}</>}
            {active.serviceType === "print-design" && activeEngine?.calculatorKey === "professional-service-v1" && programmingConfig && <PrintDesignEditor service={active} clientName={workspace.project.clientName} config={programmingConfig} pricing={pricing} suggestionsEnabled={settings.suggestionsEnabled} onChange={(config) => onProgrammingChange(active, config)} />}
            {active.serviceType !== "video-editing" && active.serviceType !== "print-design" && activeEngine?.calculatorKey === "professional-service-v1" && programmingConfig && <ProgrammingEditor service={active} config={programmingConfig} currency={workspace.quote.currency} pricing={pricing} suggestionsEnabled={settings.suggestionsEnabled} onChange={(config) => onProgrammingChange(active, config)} />}
            {productConfig && activeEngine && <ProductEditor config={productConfig} currency={workspace.quote.currency} hybrid={activeEngine.calculatorKey === "hybrid-v1"} result={activeResult} onChange={(config, immediate) => onGenericEngineChange(active, config, immediate)} />}
          </fieldset>
        </section>}
      </div>
    </main>
    <ResultInspector key={activeServiceId ?? "empty"} result={result} currency={workspace.quote.currency} activeServiceId={activeServiceId} suggestionsEnabled={settings.suggestionsEnabled} usdToArsMicros={settings.usdToArsMicros} market={market} marketJob={marketJob} onUpdateMarket={onUpdateMarket} onCancelMarket={onCancelMarket} onConfigureEconomy={onConfigureEconomy} onFinalPriceChange={active ? (final, reason) => onFinalPriceChange(active, final, reason) : undefined} />
    <footer className="actionbar">
      <div className="actionbar__summary"><FileOutput size={20} /><span>Resumen del proyecto</span><i /><strong>{workspace.services.length} {workspace.services.length === 1 ? "módulo" : "módulos"}</strong><StatusDot /></div>
      <div className="actionbar__actions">
        <Button className="actionbar__icon-action" aria-label={workspace.quote.snapshotRevision > 0 ? "Guardar revisión" : "Guardar cotización"} title={workspace.quote.snapshotRevision > 0 ? "Guardar esta revisión en el historial" : "Guardar esta cotización en el historial"} onClick={() => void onSaveQuote()}><Save size={18} /></Button>
        <Button className="actionbar__primary-action" type="button" variant="accent" onClick={() => void calculateAndReveal()} disabled={!active || calculating || isMarketUpdating}><Calculator size={18} /> {calculating ? "Calculando…" : "Calcular los 3 precios"}</Button>
        <Button className="actionbar__icon-action" type="button" aria-label={documentReady ? "Abrir presupuesto o PDF" : "Generar presupuesto o PDF"} onClick={() => void onGenerateDocument?.()} disabled={!projectReadyForDocument || !onGenerateDocument || documentBusy || isMarketUpdating} title={!projectReadyForDocument ? "Completá todos los módulos antes de preparar el presupuesto" : documentReady ? "Abrir presupuesto o PDF" : "Generar presupuesto o PDF"}><FileText size={18} /></Button>
      </div>
      <div className="actionbar__total"><span>{result.isPartial ? "Subtotal parcial" : "Total"}</span><b>{formatMoney(result.totalMinor, workspace.quote.currency)}</b></div>
    </footer>
  </div>;
}
