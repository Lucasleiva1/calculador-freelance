import { useEffect, useMemo, useState } from "react";
import {
  Activity, Archive, ArrowUpRight, Beaker, Database, Eye, FilePlus2, Pencil,
  Plus, RefreshCw, RotateCcw, Search, ShieldCheck, Trash2,
} from "lucide-react";
import type {
  ManualObservationInput, MarketObservation, MarketPriceType, MarketSource,
  MarketSourceInput, PricingConfiguration, SourceTestResult,
} from "../../domain/types";
import { majorToMinor, formatMoney } from "../../domain/money";
import { api } from "../../services/api";
import { Button, Field, Input, Modal, Select } from "../../components/ui";

type SourceAction = "edit" | "manual" | "observations" | null;

export function MarketSources({ pricing, onPricingChange }: { pricing: PricingConfiguration; onPricingChange: (pricing: PricingConfiguration) => void }) {
  const [query, setQuery] = useState("");
  const [statusFilter, setStatusFilter] = useState("all");
  const [usageFilter, setUsageFilter] = useState("all");
  const [selected, setSelected] = useState<MarketSource | null>(null);
  const [action, setAction] = useState<SourceAction>(null);
  const [creating, setCreating] = useState(false);
  const [testResult, setTestResult] = useState<SourceTestResult | null>(null);
  const [busyId, setBusyId] = useState<string | null>(null);
  const [notice, setNotice] = useState("");

  const sources = useMemo(() => pricing.marketSources.filter((source) => {
    const haystack = [source.name, source.purpose, source.dataContribution, source.appBenefit, source.sourceType].filter(Boolean).join(" ").toLowerCase();
    return haystack.includes(query.trim().toLowerCase())
      && (statusFilter === "all" || (statusFilter === "enabled" ? source.enabled : source.currentStatus === statusFilter))
      && (usageFilter === "all" || source.usageMode === usageFilter);
  }), [pricing.marketSources, query, statusFilter, usageFilter]);

  async function run(source: MarketSource, operation: () => Promise<unknown>, success: string) {
    setBusyId(source.id); setNotice("");
    try { await operation(); onPricingChange(await api.loadPricing()); setNotice(success); }
    catch (error) { setNotice(String(error)); }
    finally { setBusyId(null); }
  }

  async function test(source: MarketSource) {
    setBusyId(source.id); setNotice("");
    try { setTestResult(await api.testMarketSource(source.id)); }
    catch (error) { setNotice(String(error)); }
    finally { setBusyId(null); }
  }

  async function update(source: MarketSource, force = false) {
    if (force && !window.confirm("Esta fuente todavía está dentro del cooldown. ¿Forzar una única consulta ahora?")) return;
    setBusyId(source.id); setNotice("");
    try {
      const result = await api.refreshMarketSource(source.id, force);
      setTestResult(result); onPricingChange(await api.loadPricing());
    } catch (error) { setNotice(String(error)); }
    finally { setBusyId(null); }
  }

  function openAction(source: MarketSource, next: SourceAction) { setSelected(source); setAction(next); }

  return <div className="sources-view">
    <div className="config-toolbar source-heading"><div><span className="eyebrow">Source Registry · Fase 3</span><h2>Fuentes de mercado</h2><p>Definí qué aporta cada fuente, cómo se consulta y si puede influir en sugerencias. Las fuentes personalizadas siempre nacen manuales y sin automatización aprobada.</p></div><div className="source-heading__actions"><Button onClick={async () => { if (window.confirm("¿Restaurar y volver a mostrar todas las fuentes del catálogo? Las observaciones no se modificarán.")) onPricingChange(await api.restoreMarketSourcesCatalog()); }}><RotateCcw size={16} /> Restaurar catálogo</Button><Button variant="accent" onClick={() => setCreating(true)}><Plus size={16} /> Agregar fuente</Button></div></div>
    {notice && <div className="inline-notice" role="status" aria-live="polite">{notice}</div>}
    <div className="source-stats"><div><Database size={18} /><strong>{pricing.marketSources.length}</strong><span>registradas</span></div><div><Activity size={18} /><strong>{pricing.marketSources.filter((source) => source.enabled).length}</strong><span>activas</span></div><div><ShieldCheck size={18} /><strong>{pricing.marketSources.filter((source) => source.automationStatus === "APPROVED").length}</strong><span>auto aprobadas</span></div></div>
    <div className="list-toolbar source-filters"><label className="search-box"><Search size={17} aria-hidden="true" /><input name="market-source-search" autoComplete="off" aria-label="Buscar fuente" placeholder="Buscar fuente, dato o utilidad…" value={query} onChange={(event) => setQuery(event.target.value)} /></label><Select aria-label="Filtrar por estado" value={statusFilter} onChange={(event) => setStatusFilter(event.target.value)}><option value="all">Todos los estados</option><option value="enabled">Sólo activas</option><option value="READY">Lista</option><option value="SUCCESS">Actualizada</option><option value="MANUAL">Manual</option><option value="BLOCKED">Bloqueada</option><option value="ERROR">Error</option><option value="NEEDS_CONFIGURATION">Necesita configuración</option></Select><Select aria-label="Filtrar por uso" value={usageFilter} onChange={(event) => setUsageFilter(event.target.value)}><option value="all">Todos los usos</option><option value="market_price">Precio de mercado</option><option value="salary_context">Contexto salarial</option><option value="rate_methodology">Metodología</option><option value="currency">Moneda</option><option value="context_only">Contexto</option></Select></div>
    <div className="source-catalog">
      {sources.map((source) => <article className="source-card" key={source.id}>
        <header><div><div className="source-card__title"><h3>{source.name}</h3>{source.isSystemSource && <span className="catalog-badge">CATÁLOGO</span>}</div><button className="source-url" disabled={!source.baseUrl} onClick={() => source.baseUrl && api.openMarketSource(source.baseUrl)}>{source.baseUrl ? host(source.baseUrl) : "Sin URL"}<ArrowUpRight size={13} /></button></div><StatusBadge status={source.currentStatus} /></header>
        <div className="source-contribution"><div><span>Qué ofrece</span><p>{source.purpose || fallbackPurpose(source)}</p></div><div><span>Dato que aporta</span><p>{source.dataContribution || fallbackContribution(source)}</p></div><div><span>Cómo ayuda</span><p>{source.appBenefit || fallbackBenefit(source)}</p></div></div>
        <div className="source-meta"><span><b>Método</b>{acquisitionLabel(source.acquisitionMode)}</span><span><b>Uso</b>{usageLabel(source.usageMode)}</span><span><b>Regiones</b>{jsonList(source.regionsJson)}</span><span><b>Servicios</b>{serviceList(source.supportedServicesJson)}</span><span><b>Prioridad</b>{source.priority}</span><span><b>Cooldown</b>{source.cooldownHours ?? 24} h</span></div>
        <div className="source-health"><span>Último check <b>{relativeTime(source.lastRequestAt)}</b></span><span>Último éxito <b>{relativeTime(source.lastSuccessAt)}</b></span><span>Último fallo <b>{relativeTime(source.lastFailureAt)}</b></span><span>HTTP <b>{source.lastHttpStatus ?? "—"}</b></span><span>Fallos seguidos <b>{source.consecutiveFailures}</b></span><span>Observaciones <b>{source.observationCount}</b></span><span>Sugerencias <b>{source.participatesInSuggestions ? "Sí" : "No"}</b></span></div>
        {source.lastError && <p className="source-error">{source.lastError}</p>}
        <footer className="source-actions">
          <Button className="compact-button" disabled={busyId === source.id} onClick={() => openAction(source, "edit")}><Pencil size={14} /> Editar</Button>
          <Button className="compact-button" disabled={busyId === source.id || !source.baseUrl} onClick={() => test(source)}><Beaker size={14} /> Probar</Button>
          <Button className="compact-button" disabled={busyId === source.id || source.acquisitionMode !== "auto_http"} onClick={() => update(source, Boolean(source.cooldownUntil && new Date(source.cooldownUntil) > new Date()))}><RefreshCw className={busyId === source.id ? "spin" : ""} size={14} /> Actualizar</Button>
          <Button className="compact-button" onClick={() => openAction(source, "manual")}><FilePlus2 size={14} /> Cargar dato</Button>
          <Button className="compact-button" onClick={() => openAction(source, "observations")}><Eye size={14} /> Ver datos</Button>
          {source.isSystemSource && <Button className="compact-button" aria-label={`Restaurar ${source.name}`} title="Restaurar configuración de catálogo" onClick={() => run(source, () => api.restoreMarketSource(source.id), "Fuente restaurada.")}><RotateCcw size={14} aria-hidden="true" /></Button>}
          <Button variant="danger" className="compact-button" aria-label={`Quitar ${source.name}`} title="Quitar fuente del registro" onClick={() => { if (window.confirm(`¿Quitar “${source.name}” del registro? Sus observaciones históricas se conservarán para auditoría.`)) void run(source, () => api.deleteMarketSource(source.id), "Fuente quitada; su historial se conservó."); }}><Trash2 size={14} aria-hidden="true" /></Button>
        </footer>
      </article>)}
      {sources.length === 0 && <div className="source-empty"><Archive size={28} /><strong>No hay fuentes para estos filtros</strong><span>Probá otra búsqueda o agregá una fuente propia.</span></div>}
    </div>
    {(creating || (action === "edit" && selected)) && <SourceModal source={creating ? null : selected} onClose={() => { setCreating(false); setSelected(null); setAction(null); }} onSave={async (input) => { onPricingChange(await api.saveMarketSource(input)); setCreating(false); setSelected(null); setAction(null); setNotice("Fuente guardada."); }} />}
    {testResult && <TestResultModal result={testResult} source={pricing.marketSources.find((item) => item.id === testResult.sourceId) ?? null} onClose={() => setTestResult(null)} onApprove={async (id) => { onPricingChange(await api.approveMarketSource(id)); setTestResult(null); setNotice("Fuente aprobada para una consulta HTTP conservadora."); }} />}
    {action === "manual" && selected && <ManualObservationModal source={selected} onClose={() => { setSelected(null); setAction(null); }} onSaved={async () => { onPricingChange(await api.loadPricing()); setSelected(null); setAction(null); setNotice("Observación manual guardada con trazabilidad."); }} />}
    {action === "observations" && selected && <SourceObservationsModal source={selected} onClose={() => { setSelected(null); setAction(null); }} />}
  </div>;
}

function StatusBadge({ status }: { status: MarketSource["currentStatus"] }) { return <span className={`source-status source-status--${status.toLowerCase()}`}>{statusLabel(status)}</span>; }
function label(value: string) { return value.replaceAll("_", " ").toLocaleLowerCase("es-AR"); }
function statusLabel(value: MarketSource["currentStatus"]) { return ({ READY: "Lista", FETCHING: "Consultando", SUCCESS: "Actualizada", CACHED: "En caché", MANUAL: "Manual", BLOCKED: "Bloqueada", ERROR: "Error", DISABLED: "Desactivada", NEEDS_CONFIGURATION: "Necesita configuración" } as const)[value]; }
function acquisitionLabel(value: MarketSource["acquisitionMode"]) { return ({ auto_http: "HTTP automático", auto_browser: "Navegador aislado", manual: "Manual", disabled: "Desactivado" } as const)[value]; }
function usageLabel(value: string) { return ({ market_price: "Precio de mercado", salary_context: "Contexto salarial", rate_methodology: "Metodología", currency: "Moneda", context_only: "Sólo contexto" } as Record<string, string>)[value] ?? label(value); }
function host(raw: string) { try { return new URL(raw).hostname; } catch { return raw; } }
function relativeTime(raw: string | null) { if (!raw) return "Nunca"; const hours = Math.floor((Date.now() - new Date(raw).getTime()) / 3_600_000); return hours < 1 ? "Hace menos de 1 h" : hours < 48 ? `Hace ${hours} h` : new Date(raw).toLocaleDateString("es-AR"); }
function jsonList(value: string) { try { return (JSON.parse(value) as string[]).join(" · ") || "Global"; } catch { return "—"; } }
function serviceList(value: string) { return jsonList(value).replace("video-editing", "Video").replace("programming", "Programación"); }
function fallbackPurpose(source: MarketSource) { return source.usageMode === "salary_context" ? "Contexto salarial separado de las tarifas freelance." : "Referencia pública configurable para investigación de mercado."; }
function fallbackContribution(source: MarketSource) { return source.usageMode === "currency" ? "Cotización, par, fecha y organismo." : "Precio o rango, moneda, unidad, región y fecha."; }
function fallbackBenefit(source: MarketSource) { return source.usageMode === "salary_context" ? "Ayuda a entender el contexto laboral sin contaminar la mediana freelance." : "Contrasta el cálculo interno manteniendo el precio final bajo tu control."; }

function SourceModal({ source, onClose, onSave }: { source: MarketSource | null; onClose: () => void; onSave: (input: MarketSourceInput) => Promise<void> }) {
  const [form, setForm] = useState({
    name: source?.name ?? "", url: source?.baseUrl ?? "", type: source?.sourceType ?? "other",
    regions: source ? jsonList(source.regionsJson).replaceAll(" · ", ", ") : "GLOBAL",
    services: source ? jsonList(source.supportedServicesJson).replaceAll(" · ", ", ") : "video-editing",
    priority: source?.priority ?? 100, enabled: source?.enabled ?? true, usage: source?.usageMode ?? "context_only",
    acquisition: (source?.acquisitionMode ?? "manual") as MarketSource["acquisitionMode"], cooldown: source?.cooldownHours?.toString() ?? "24",
    participates: source?.participatesInSuggestions ?? false, purpose: source?.purpose ?? "",
    contribution: source?.dataContribution ?? "", benefit: source?.appBenefit ?? "", notes: source?.notes ?? "",
  });
  const [error, setError] = useState(""); const [saving, setSaving] = useState(false);
  const listJson = (value: string) => JSON.stringify(value.split(",").map((item) => item.trim()).filter(Boolean));
  const autoAllowed = source?.automationStatus === "APPROVED";
  return <Modal title={source ? `Editar · ${source.name}` : "Agregar fuente"} onClose={onClose} width="920px"><form className="modal__body form-stack source-form" onSubmit={async (event) => { event.preventDefault(); setSaving(true); setError(""); try { await onSave({ id: source?.id, name: form.name, baseUrl: form.url, sourceType: form.type, regionsJson: listJson(form.regions), supportedServicesJson: listJson(form.services), priority: Number(form.priority), enabled: form.enabled, usageMode: form.usage, acquisitionMode: source ? form.acquisition as MarketSource["acquisitionMode"] : "manual", cooldownHours: form.cooldown ? Number(form.cooldown) : 24, notes: form.notes, purpose: form.purpose, dataContribution: form.contribution, appBenefit: form.benefit, participatesInSuggestions: form.participates }); } catch (err) { setError(String(err)); } finally { setSaving(false); } }}>
    {!source && <div className="security-note"><ShieldCheck size={18} /><div><strong>Alta segura</strong><span>La fuente se guardará como MANUAL + UNREVIEWED. Después podrás probar una sola extracción y aprobarla explícitamente.</span></div></div>}
    <div className="form-grid"><Field label="Nombre"><Input required value={form.name} onChange={(event) => setForm({ ...form, name: event.target.value })} /></Field><Field label="URL base · HTTPS"><Input required type="url" pattern="https://.*" value={form.url} onChange={(event) => setForm({ ...form, url: event.target.value })} /></Field><Field label="Tipo"><Select value={form.type} onChange={(event) => setForm({ ...form, type: event.target.value })}>{typeOptions.map(([value, text]) => <option key={value} value={value}>{text}</option>)}</Select></Field><Field label="Uso en Pricing OS"><Select value={form.usage} onChange={(event) => setForm({ ...form, usage: event.target.value, participates: event.target.value === "market_price" ? form.participates : false })}>{usageOptions.map(([value, text]) => <option key={value} value={value}>{text}</option>)}</Select></Field><Field label="Regiones · separadas por coma" hint="Ej.: AR, LATAM, INTERNATIONAL"><Input value={form.regions} onChange={(event) => setForm({ ...form, regions: event.target.value })} /></Field><Field label="Servicios · separados por coma" hint="video-editing, programming"><Input value={form.services} onChange={(event) => setForm({ ...form, services: event.target.value })} /></Field><Field label="Método de adquisición"><Select disabled={!source} value={form.acquisition} onChange={(event) => setForm({ ...form, acquisition: event.target.value as MarketSource["acquisitionMode"] })}><option value="manual">Manual</option><option value="auto_http" disabled={!autoAllowed}>AUTO_HTTP{!autoAllowed ? " · requiere prueba" : ""}</option><option value="auto_browser">AUTO_BROWSER · aislado</option><option value="disabled">Desactivado</option></Select></Field><Field label="Cooldown · horas"><Input type="number" min="0" max="720" value={form.cooldown} onChange={(event) => setForm({ ...form, cooldown: event.target.value })} /></Field><Field label="Prioridad"><Input type="number" min="0" value={form.priority} onChange={(event) => setForm({ ...form, priority: Number(event.target.value) })} /></Field><div className="source-checks"><label className="check-label"><input type="checkbox" checked={form.enabled} onChange={(event) => setForm({ ...form, enabled: event.target.checked })} /> Activa</label><label className="check-label" title={form.usage !== "market_price" ? "Las fuentes de salario, metodología, moneda y contexto se excluyen por diseño." : undefined}><input type="checkbox" disabled={form.usage !== "market_price"} checked={form.participates} onChange={(event) => setForm({ ...form, participates: event.target.checked })} /> Participa en sugerencias</label></div><Field label="Qué ofrece esta fuente" className="span-2"><textarea required className="input textarea" value={form.purpose} onChange={(event) => setForm({ ...form, purpose: event.target.value })} /></Field><Field label="Qué dato aporta específicamente" className="span-2"><textarea required className="input textarea" value={form.contribution} onChange={(event) => setForm({ ...form, contribution: event.target.value })} /></Field><Field label="Cómo ayuda a Pricing OS" className="span-2"><textarea required className="input textarea" value={form.benefit} onChange={(event) => setForm({ ...form, benefit: event.target.value })} /></Field><Field label="Notas internas · opcional" className="span-2"><textarea className="input textarea" value={form.notes} onChange={(event) => setForm({ ...form, notes: event.target.value })} /></Field></div>
    {error && <div className="form-error">{error}</div>}<div className="modal__actions"><Button type="button" onClick={onClose}>Cancelar</Button><Button variant="accent" disabled={saving}>{saving ? "Guardando…" : "Guardar fuente"}</Button></div>
  </form></Modal>;
}

function TestResultModal({ result, source, onClose, onApprove }: { result: SourceTestResult; source: MarketSource | null; onClose: () => void; onApprove: (id: string) => Promise<void> }) {
  const canApprove = result.status === "SUCCESS" && source?.automationStatus === "UNREVIEWED";
  return <Modal title="Resultado de prueba" onClose={onClose} width="780px"><div className="modal__body test-result"><StatusBadge status={result.status} /><p>{result.message}</p>{result.httpStatus && <span className="muted-line">HTTP {result.httpStatus}</span>}<div className="preview-list">{result.observations.map((item, index) => <div key={`${item.subservice}-${index}`}><span>{item.subservice || item.serviceType}</span><strong>{previewMoney(item)}</strong><small>{item.priceType.replaceAll("_", " ")} · {item.unit} · {item.region}</small>{item.evidence && <p>{item.evidence}</p>}</div>)}</div>{result.observations.length === 0 && <div className="source-empty"><Beaker size={24} /><span>No se guardó ningún dato definitivo.</span></div>}<div className="modal__actions"><Button onClick={onClose}>Descartar</Button>{canApprove && <Button variant="accent" onClick={() => onApprove(result.sourceId)}><ShieldCheck size={16} /> Guardar configuración y aprobar</Button>}</div></div></Modal>;
}

function previewMoney(item: SourceTestResult["observations"][number]) { const value = item.priceValueMinor ?? (item.priceMinMinor != null && item.priceMaxMinor != null ? Math.round((item.priceMinMinor + item.priceMaxMinor) / 2) : item.priceMinMinor ?? item.priceMaxMinor); return `${item.currency} ${value == null ? "—" : (value / 100).toLocaleString("es-AR", { maximumFractionDigits: 2 })}`; }

function ManualObservationModal({ source, onClose, onSaved }: { source: MarketSource; onClose: () => void; onSaved: () => Promise<void> }) {
  const [form, setForm] = useState({ service: "video-editing", subservice: "", category: "", min: "", max: "", value: "", currency: "USD", priceType: "PROJECT" as MarketPriceType, unit: "por proyecto", region: jsonValues(source.regionsJson)[0] ?? "GLOBAL", country: "", experience: "", clientTier: "", date: new Date().toISOString().slice(0, 10), url: source.baseUrl ?? "", notes: "" });
  const [error, setError] = useState(""); const [saving, setSaving] = useState(false);
  return <Modal title={`Cargar observación · ${source.name}`} onClose={onClose} width="880px"><form className="modal__body form-stack" onSubmit={async (event) => { event.preventDefault(); setSaving(true); setError(""); try { const input: ManualObservationInput = { sourceId: source.id, serviceType: form.service, subservice: form.subservice, category: form.category, region: form.region, country: form.country, currency: form.currency, priceType: form.priceType, unit: form.unit, priceMinMinor: majorToMinor(form.min), priceMaxMinor: majorToMinor(form.max), priceValueMinor: majorToMinor(form.value), experienceLevel: form.experience, clientTier: form.clientTier, publishedAt: form.date, sourceUrl: form.url, notes: form.notes }; await api.saveManualMarketObservation(input); await onSaved(); } catch (err) { setError(String(err)); } finally { setSaving(false); } }}><div className="security-note"><FilePlus2 size={18} /><div><strong>Misma trazabilidad que una observación automática</strong><span>Guardará fuente, URL, fecha, unidad, moneda, origen MANUAL y fingerprint anti-duplicados.</span></div></div><div className="form-grid"><Field label="Servicio"><Select value={form.service} onChange={(event) => setForm({ ...form, service: event.target.value })}><option value="video-editing">Edición de video</option><option value="programming">Programación</option></Select></Field><Field label="Subservicio / rol"><Input required value={form.subservice} onChange={(event) => setForm({ ...form, subservice: event.target.value })} /></Field><Field label="Precio puntual"><Input type="number" min="0" step="0.01" value={form.value} onChange={(event) => setForm({ ...form, value: event.target.value })} /></Field><Field label="Precio mínimo"><Input type="number" min="0" step="0.01" value={form.min} onChange={(event) => setForm({ ...form, min: event.target.value })} /></Field><Field label="Precio máximo"><Input type="number" min="0" step="0.01" value={form.max} onChange={(event) => setForm({ ...form, max: event.target.value })} /></Field><Field label="Moneda"><Select value={form.currency} onChange={(event) => setForm({ ...form, currency: event.target.value })}><option>USD</option><option>ARS</option><option>GBP</option><option>EUR</option></Select></Field><Field label="Tipo de precio"><Select value={form.priceType} onChange={(event) => setForm({ ...form, priceType: event.target.value as MarketPriceType })}>{priceTypeOptions.map((value) => <option key={value}>{value}</option>)}</Select></Field><Field label="Unidad"><Input required value={form.unit} onChange={(event) => setForm({ ...form, unit: event.target.value })} /></Field><Field label="Región"><Input required value={form.region} onChange={(event) => setForm({ ...form, region: event.target.value })} /></Field><Field label="País"><Input value={form.country} onChange={(event) => setForm({ ...form, country: event.target.value })} /></Field><Field label="Experiencia"><Input value={form.experience} onChange={(event) => setForm({ ...form, experience: event.target.value })} /></Field><Field label="Tipo de cliente"><Input value={form.clientTier} onChange={(event) => setForm({ ...form, clientTier: event.target.value })} /></Field><Field label="Fecha publicada"><Input type="date" required value={form.date} onChange={(event) => setForm({ ...form, date: event.target.value })} /></Field><Field label="URL exacta de evidencia" className="span-2"><Input required type="url" pattern="https://.*" value={form.url} onChange={(event) => setForm({ ...form, url: event.target.value })} /></Field><Field label="Nota" className="span-2"><textarea className="input textarea" value={form.notes} onChange={(event) => setForm({ ...form, notes: event.target.value })} /></Field></div><p className="muted-line">Completá un precio puntual o un rango mínimo/máximo. Los salarios mensuales y anuales quedarán fuera de la mediana freelance.</p>{error && <div className="form-error">{error}</div>}<div className="modal__actions"><Button type="button" onClick={onClose}>Cancelar</Button><Button variant="accent" disabled={saving}>{saving ? "Guardando…" : "Guardar observación"}</Button></div></form></Modal>;
}

function SourceObservationsModal({ source, onClose }: { source: MarketSource; onClose: () => void }) {
  const [rows, setRows] = useState<MarketObservation[] | null>(null); const [error, setError] = useState("");
  useEffect(() => { void api.listMarketObservations({ sourceId: source.id }).then(setRows).catch((reason) => setError(String(reason))); }, [source.id]);
  return <Modal title={`Observaciones · ${source.name}`} onClose={onClose} width="1040px"><div className="modal__body observations-modal">{error && <div className="form-error" role="alert">{error}</div>}{rows === null ? <p>Cargando…</p> : rows.length === 0 ? <div className="source-empty"><Database size={25} /><strong>Todavía no hay observaciones</strong><span>Actualizá la fuente o cargá un dato manual.</span></div> : <div className="observation-table" role="table" aria-label={`Observaciones de ${source.name}`} tabIndex={0}><div className="observation-row observation-row--head" role="row"><span role="columnheader">Servicio</span><span role="columnheader">Precio</span><span role="columnheader">Unidad</span><span role="columnheader">Fecha</span><span role="columnheader">Estado</span><span role="columnheader" /></div>{rows.map((row) => <div className="observation-row" role="row" key={row.id}><span role="cell"><strong>{row.subservice || row.serviceType}</strong><small>{row.region} · {row.origin}</small></span><span role="cell">{observationMoney(row)}</span><span role="cell">{row.priceType.replaceAll("_", " ")}<small>{row.unit}</small></span><span role="cell">{new Date(row.publishedAt || row.retrievedAt).toLocaleDateString("es-AR")}</span><span role="cell"><b>{row.comparisonEligibility === "ELIGIBLE" ? "Comparable" : "Contexto"}</b><small>{row.exclusionReason}</small></span><button type="button" aria-label={`Abrir fuente original de ${row.subservice || source.name}`} onClick={() => api.openMarketSource(row.sourceUrl)}><ArrowUpRight size={15} aria-hidden="true" /></button></div>)}</div>}<div className="modal__actions"><Button onClick={onClose}>Cerrar</Button></div></div></Modal>;
}

function observationMoney(row: MarketObservation) { if (row.priceValueMinor != null) return formatMoney(row.priceValueMinor, row.currency as "ARS" | "USD"); const min = row.priceMinMinor == null ? "—" : (row.priceMinMinor / 100).toLocaleString("es-AR"); const max = row.priceMaxMinor == null ? "—" : (row.priceMaxMinor / 100).toLocaleString("es-AR"); return `${row.currency} ${min}–${max}`; }
function jsonValues(value: string) { try { return JSON.parse(value) as string[]; } catch { return []; } }

const typeOptions = [["freelance_marketplace", "Marketplace freelance"], ["rate_benchmark", "Benchmark de tarifas"], ["professional_tariff", "Tarifario profesional"], ["salary", "Salarios"], ["job_board", "Bolsa de trabajo"], ["agency_pricing", "Precios de agencia"], ["methodology", "Metodología"], ["currency", "Moneda"], ["other", "Otra"]] as const;
const usageOptions = [["market_price", "Precio de mercado"], ["salary_context", "Contexto salarial"], ["rate_methodology", "Metodología de tarifa"], ["currency", "Moneda"], ["context_only", "Sólo contexto"]] as const;
const priceTypeOptions: MarketPriceType[] = ["HOURLY", "DAILY", "PROJECT", "PER_MINUTE", "PER_ITEM", "MONTHLY_SALARY", "ANNUAL_SALARY", "FIXED", "RANGE", "UNKNOWN"];
