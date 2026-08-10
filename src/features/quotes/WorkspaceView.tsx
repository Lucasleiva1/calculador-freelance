import { useEffect, useMemo, useState } from "react";
import { ArrowDown, ArrowUp, Blend, Code2, Film, FileOutput, Package, Plus, Trash2 } from "lucide-react";
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
import { DynamicFields } from "../../components/DynamicFields";
import type { ProjectResult } from "../../domain/quote";
import { formatMoney } from "../../domain/money";
import { VideoEditor } from "../video/VideoEditor";
import { ProgrammingEditor } from "../programming/ProgrammingEditor";
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
}) {
  const [addOpen, setAddOpen] = useState(false);
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
    try { return parseProgrammingEnvelope(active.configurationJson).data; }
    catch { return { parameterValues: {}, externalCosts: [], notes: "" }; }
  }, [active, activeEngine?.calculatorKey]);
  const productConfig = useMemo(() => {
    if (!active || !activeEngine || !["physical-product-v1", "hybrid-v1"].includes(activeEngine.calculatorKey)) return null;
    try { return (JSON.parse(active.configurationJson) as ServiceConfigurationEnvelope<ProductConfiguration | HybridConfiguration>).data; }
    catch { return null; }
  }, [active, activeEngine]);
  const activeEngines = pricing.pricingEngines.filter((engine) => engine.status === "active" && engine.calculatorKey !== "unconfigured");

  function EngineIcon({ type, size = 16 }: { type: string; size?: number }) {
    if (type === "video-editing") return <Film size={size} />;
    const engine = pricing.pricingEngines.find((item) => item.engineKey === type);
    if (engine?.engineType === "product") return <Package size={size} />;
    if (engine?.engineType === "hybrid") return <Blend size={size} />;
    return <Code2 size={size} />;
  }

  return <div className="workspace-layout">
    <main className="workspace-main">
      <div className="workspace-scroll">
        <header className="page-header"><div><span className="eyebrow">Cotización · Draft v{workspace.quote.version}</span><h1>Cotización</h1><p>Construí el proyecto por módulos independientes.</p></div><div className={`save-indicator save-indicator--${activeStatus ?? "saved"}`}><StatusDot tone={activeStatus === "error" ? "danger" : activeStatus === "saving" ? "muted" : "accent"} />{saveLabel(activeStatus)}{activeStatus === "error" && active && <button onClick={() => onRetry(active.id)}>Reintentar</button>}</div></header>
        <div className="service-tabs">
          {workspace.services.map((service, index) => <button key={service.id} className={active?.id === service.id ? "is-active" : ""} onClick={() => onActiveService(service.id)}><span>{String(index + 1).padStart(2, "0")}</span><EngineIcon type={service.serviceType} /><strong>{service.title}</strong></button>)}
          <div className="add-service"><button className="add-service__trigger" onClick={() => setAddOpen(!addOpen)}><Plus size={17} /> Agregar módulo</button>{addOpen && <div className="add-service__menu">{activeEngines.map((engine) => <button key={engine.id} onClick={async () => { setAddOpen(false); await onAddService(engine.engineKey); }}><EngineIcon type={engine.engineKey} size={17} /><span>{engine.name}<small>{engine.engineType === "product" ? "Producto físico" : engine.engineType === "hybrid" ? "Servicio + producto" : engine.engineKey === "video-editing" ? "Editor audiovisual" : "Servicio profesional"}</small></span></button>)}<button onClick={() => { setAddOpen(false); }} disabled><Plus size={17} /><span>Nuevo motor<small>Crealo desde Servicios</small></span></button></div>}</div>
        </div>

        {!active ? <EmptyState eyebrow="Cotización vacía" title="Agregá el primer módulo" description="Los servicios, productos e híbridos viven como motores independientes dentro del mismo proyecto." action={<Button variant="accent" onClick={() => onAddService("video-editing")}><Plus size={17} /> Agregar Edición de video</Button>} /> : <section className="service-panel">
          <header className="service-panel__header"><div className="service-title"><span>{String(workspace.services.indexOf(active) + 1).padStart(2, "0")} /</span><input aria-label="Título del módulo" value={active.title} onChange={(event) => onTitleChange(active, event.target.value)} /></div><div className="service-panel__tools"><span>{formatMoney(activeResult?.effectiveSubtotalMinor, workspace.quote.currency)}</span><button title="Mover arriba" disabled={workspace.services.indexOf(active) === 0} onClick={() => onMoveService(active, -1)}><ArrowUp size={16} /></button><button title="Mover abajo" disabled={workspace.services.indexOf(active) === workspace.services.length - 1} onClick={() => onMoveService(active, 1)}><ArrowDown size={16} /></button><button title="Quitar módulo" onClick={() => onDeleteService(active)}><Trash2 size={16} /></button></div></header>
          {activeStatus === "error" && <div className="save-error" role="alert">{errors[active.id]}</div>}
          {active.serviceType === "video-editing" && videoConfig && <><VideoEditor service={active} config={videoConfig} currency={workspace.quote.currency} presets={presets.filter((preset) => preset.serviceType === "video-editing")} onChange={(config, manual, reason, immediate) => onVideoChange(active, config, manual, reason, immediate)} onSavePreset={onSavePreset} onUpdatePreset={onUpdatePreset} onDeletePreset={onDeletePreset} onRestorePreset={onRestorePreset} />{pricing.parameters.some((item) => item.serviceDefinitionId === pricing.definitions.find((definition) => definition.serviceType === "video-editing")?.id && !item.uiManaged && item.enabled) && <section className="editor-section"><span className="eyebrow">Parámetros personalizados</span><DynamicFields parameters={pricing.parameters.filter((item) => item.serviceDefinitionId === pricing.definitions.find((definition) => definition.serviceType === "video-editing")?.id && !item.uiManaged)} options={pricing.options} values={videoConfig as unknown as Record<string, unknown>} suggestionsEnabled={settings.suggestionsEnabled} onChange={(values) => onVideoChange(active, values as unknown as VideoConfiguration)} /></section>}</>}
          {active.serviceType !== "video-editing" && activeEngine?.calculatorKey === "professional-service-v1" && programmingConfig && <ProgrammingEditor service={active} config={programmingConfig} currency={workspace.quote.currency} pricing={pricing} suggestionsEnabled={settings.suggestionsEnabled} onChange={(config) => onProgrammingChange(active, config)} />}
          {productConfig && activeEngine && <ProductEditor config={productConfig} currency={workspace.quote.currency} hybrid={activeEngine.calculatorKey === "hybrid-v1"} result={activeResult} onChange={(config, immediate) => onGenericEngineChange(active, config, immediate)} />}
        </section>}
      </div>
    </main>
    <ResultInspector key={activeServiceId ?? "empty"} result={result} currency={workspace.quote.currency} activeServiceId={activeServiceId} suggestionsEnabled={settings.suggestionsEnabled} market={market} marketJob={marketJob} onUpdateMarket={onUpdateMarket} onCancelMarket={onCancelMarket} onFinalPriceChange={active ? (final, reason) => onFinalPriceChange(active, final, reason) : undefined} />
    <footer className="actionbar"><div className="actionbar__summary"><FileOutput size={20} /><span>Resumen del proyecto</span><i /><strong>{workspace.services.length} {workspace.services.length === 1 ? "módulo" : "módulos"}</strong><StatusDot /><i /><span>{result.isPartial ? "Subtotal parcial" : "Total"}</span><b>{formatMoney(result.totalMinor, workspace.quote.currency)}</b></div><div className="actionbar__actions"><Button disabled>Guardar borrador</Button><Button variant="accent" disabled>Generar presupuesto · Próximamente</Button><Button disabled>Exportar / PDF</Button></div></footer>
  </div>;
}
