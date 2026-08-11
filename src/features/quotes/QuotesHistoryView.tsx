import { useEffect, useMemo, useState } from "react";
import { Archive, ArrowRight, CalendarDays, Copy, FileClock, Filter, Pencil, RefreshCw, RotateCcw, Search, Trash2, X } from "lucide-react";
import { formatMoney, minorToInput, majorToMinor } from "../../domain/money";
import { filterQuoteHistory, parseQuoteSnapshot, quoteStatusLabels, type QuoteHistoryFilters } from "../../domain/quoteHistory";
import type { Client, PricingConfiguration, QuoteHistoryDetail, QuoteHistoryItem, QuotePriceKind, QuoteStatus, Workspace } from "../../domain/types";
import { api } from "../../services/api";
import { Button, EmptyState, Field, Input, Select, StatusDot } from "../../components/ui";

const initialFilters: QuoteHistoryFilters = { query: "", status: "all", serviceType: "all", currency: "all", sort: "recent" };

function formatDate(value: string) {
  return new Intl.DateTimeFormat("es-AR", { dateStyle: "medium", timeStyle: "short" }).format(new Date(value));
}

function Detail({ detail, clients, onClose, onReload, onOpenProject, onDuplicated }: {
  detail: QuoteHistoryDetail; clients: Client[]; onClose: () => void;
  onReload: (quoteId: string, revision?: number) => Promise<void>;
  onOpenProject: (projectId: string) => Promise<void>;
  onDuplicated: (workspace: Workspace) => void;
}) {
  const snapshot = parseQuoteSnapshot(detail.snapshotJson);
  const historicalRevision = detail.displayedRevision !== detail.quote.snapshotRevision;
  const currency = snapshot?.quote.currency ?? detail.quote.currency;
  const displayedSelectedMinor = historicalRevision ? snapshot?.totals.selectedMinor ?? null : detail.quote.selectedPriceMinor;
  const displayedSelectedKind = historicalRevision ? snapshot?.quote.selectedPriceKind ?? detail.quote.selectedPriceKind : detail.quote.selectedPriceKind;
  const priceKindLabel: Record<QuotePriceKind, string> = { floor: "Piso", recommended: "Recomendado", premium: "Premium", custom: "Personalizado" };
  const [editing, setEditing] = useState(false);
  const [projectName, setProjectName] = useState(detail.quote.projectName);
  const [clientId, setClientId] = useState(detail.quote.clientId);
  const [notes, setNotes] = useState(detail.quote.notes ?? "");
  const [status, setStatus] = useState<QuoteStatus>(detail.quote.status);
  const [kind, setKind] = useState<QuotePriceKind>(detail.quote.selectedPriceKind);
  const [custom, setCustom] = useState(minorToInput(detail.quote.selectedPriceKind === "custom" ? detail.quote.selectedPriceMinor : null));
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState("");

  async function saveAdmin() {
    setBusy(true); setError("");
    try {
      await api.updateQuoteAdmin({ quoteId: detail.quote.id, projectName, clientId, notes, status, selectedPriceKind: kind, selectedPriceMinor: kind === "custom" ? majorToMinor(custom) : detail.quote.selectedPriceMinor });
      setEditing(false); await onReload(detail.quote.id, detail.displayedRevision);
    } catch (reason) { setError(String(reason)); }
    finally { setBusy(false); }
  }

  async function duplicate() {
    if (!window.confirm("Se creará un proyecto nuevo con los mismos módulos, parámetros y precios guardados. No se recalculará nada. ¿Continuar?")) return;
    setBusy(true); setError("");
    try { onDuplicated(await api.duplicateQuote({ quoteId: detail.quote.id, revision: detail.displayedRevision })); }
    catch (reason) { setError(String(reason)); setBusy(false); }
  }

  async function changeArchive(archived: boolean) {
    setBusy(true); setError("");
    try {
      await api.updateQuoteAdmin({ quoteId: detail.quote.id, projectName, clientId, notes, status: archived ? "archived" : "draft", selectedPriceKind: kind, selectedPriceMinor: kind === "custom" ? majorToMinor(custom) : detail.quote.selectedPriceMinor });
      await onReload(detail.quote.id);
    } catch (reason) { setError(String(reason)); }
    finally { setBusy(false); }
  }

  async function permanentDelete() {
    if (!window.confirm(`Eliminar definitivamente “${detail.quote.projectName}” y todas sus revisiones? Esta acción no se puede deshacer.`)) return;
    setBusy(true);
    try { await api.deleteQuotePermanently(detail.quote.id); onClose(); }
    catch (reason) { setError(String(reason)); setBusy(false); }
  }

  return <aside className="quote-detail" aria-label="Detalle histórico de cotización">
    <header className="quote-detail__header"><div><span className="eyebrow">Snapshot histórico</span><h2>{detail.quote.projectName}</h2><p>{detail.quote.clientName} · {currency}</p></div><button className="icon-button" aria-label="Cerrar detalle" onClick={onClose}><X size={19} /></button></header>
    <div className="quote-detail__toolbar"><Select aria-label="Revisión histórica" value={detail.displayedRevision} onChange={(event) => void onReload(detail.quote.id, Number(event.target.value))}>{detail.revisions.map((revision) => <option key={revision.revision} value={revision.revision}>Revisión {revision.revision} · {formatDate(revision.createdAt)}</option>)}</Select><span className={`quote-status quote-status--${detail.quote.status}`}><StatusDot tone={detail.quote.status === "rejected" ? "danger" : detail.quote.status === "archived" ? "muted" : "accent"} />{quoteStatusLabels[detail.quote.status]}</span></div>
    <div className="quote-detail__scroll">
      {historicalRevision && <p className="historical-warning"><FileClock size={16} />Estás viendo una revisión anterior e inmutable.</p>}
      {editing ? <section className="quote-admin-form"><span className="eyebrow">Datos administrativos</span><Field label="Proyecto" hint="Cambiar este nombre no recalcula los importes históricos."><Input value={projectName} onChange={(event) => setProjectName(event.target.value)} /></Field><Field label="Cliente" hint="Asocia la cotización a otro cliente existente; el snapshot visto conserva el cliente de aquella fecha."><Select value={clientId} onChange={(event) => setClientId(event.target.value)}>{clients.filter((client) => client.status === "active").map((client) => <option key={client.id} value={client.id}>{client.name}</option>)}</Select></Field><Field label="Estado" hint="Sirve para seguir el ciclo comercial sin convertir Pricing OS en un CRM."><Select value={status} onChange={(event) => setStatus(event.target.value as QuoteStatus)}>{Object.entries(quoteStatusLabels).map(([value, label]) => <option key={value} value={value}>{label}</option>)}</Select></Field><Field label="Notas" hint="Alcance y condiciones administrativas actuales."><textarea className="input textarea" rows={3} value={notes} onChange={(event) => setNotes(event.target.value)} /></Field><Field label="Precio seleccionado" hint="Podés cambiar la decisión final sin alterar piso, recomendado ni premium."><Select value={kind} onChange={(event) => setKind(event.target.value as QuotePriceKind)}><option value="floor">Piso</option><option value="recommended">Recomendado</option><option value="premium">Premium</option><option value="custom">Personalizado</option></Select></Field>{kind === "custom" && <Field label={`Importe (${detail.quote.currency})`}><Input inputMode="decimal" value={custom} onChange={(event) => setCustom(event.target.value)} /></Field>}<div className="quote-admin-form__actions"><Button onClick={() => setEditing(false)}>Cancelar</Button><Button variant="accent" onClick={() => void saveAdmin()} disabled={busy}>Guardar datos</Button></div></section> : <>
        <section className="quote-detail__prices"><div><span>Piso</span><strong>{formatMoney(snapshot?.totals.floorMinor, currency)}</strong></div><div className="is-featured"><span>Recomendado</span><strong>{formatMoney(snapshot?.totals.recommendedMinor, currency)}</strong></div><div><span>Premium</span><strong>{formatMoney(snapshot?.totals.premiumMinor, currency)}</strong></div><div className="is-selected"><span>Precio seleccionado</span><strong>{formatMoney(displayedSelectedMinor, currency)}</strong><small>{priceKindLabel[displayedSelectedKind]}</small></div></section>
        <section className="quote-detail__metrics"><div><span>Horas estimadas</span><b>{snapshot ? (snapshot.totals.totalHoursMicros / 1_000_000).toLocaleString("es-AR") : "—"} h</b></div><div><span>Costos externos</span><b>{formatMoney(snapshot?.totals.externalCostsMinor, currency)}</b></div><div><span>Valor efectivo / h</span><b>{formatMoney(snapshot?.totals.effectiveHourlyMinor, currency)}</b></div><div><span>Margen registrado</span><b>{snapshot?.totals.marginMicros == null ? "—" : `${(snapshot.totals.marginMicros / 10_000).toLocaleString("es-AR")}%`}</b></div></section>
        {snapshot?.quote.notes && <section className="quote-detail__notes"><span className="eyebrow">Notas</span><p>{snapshot.quote.notes}</p></section>}
        <section className="quote-detail__services"><span className="eyebrow">Módulos congelados</span>{snapshot?.services.map((service, index) => <article key={service.id}><header><span>{String(index + 1).padStart(2, "0")}</span><div><strong>{service.title}</strong><small>{service.serviceType}</small></div><b>{formatMoney(service.finalSubtotalMinor, currency)}</b></header><div className="snapshot-service__facts"><span>Calculado <b>{formatMoney(service.calculatedSubtotalMinor, currency)}</b></span><span>Sugerido <b>{formatMoney(service.suggestedSubtotalMinor, currency)}</b></span>{service.hasOverride && <span>Override <b>{formatMoney(service.manualSubtotalMinor, currency)}</b></span>}</div>{service.manualReason && <p>{service.manualReason}</p>}<details><summary>Parámetros y fuentes</summary><pre>{JSON.stringify(service.configuration, null, 2)}</pre><div className="snapshot-sources">{service.sources.assigned.length === 0 ? <small>Sin fuentes asignadas en este corte.</small> : service.sources.assigned.map((source) => <div key={source.id}><strong>{source.name}</strong><span>{source.contribution || source.role}</span>{source.url && <small>{source.url}</small>}</div>)}</div></details></article>) ?? <p>No se pudo leer el snapshot.</p>}</section>
      </>}
      {error && <p className="form-error" role="alert">{error}</p>}
    </div>
    <footer className="quote-detail__actions"><Button onClick={() => setEditing(!editing)}><Pencil size={15} /> Editar datos</Button><Button onClick={() => void duplicate()} disabled={busy}><Copy size={15} /> Usar como base</Button><Button onClick={() => { if (window.confirm("Abrir el cálculo permite modificar el borrador vivo. Los snapshots anteriores seguirán intactos y sólo se actualizará el historial cuando guardes una nueva revisión. ¿Abrir?")) void onOpenProject(detail.quote.projectId); }}><ArrowRight size={15} /> Editar cálculo</Button>{detail.quote.status === "archived" ? <><Button onClick={() => void changeArchive(false)}><RotateCcw size={15} /> Restaurar</Button><Button variant="danger" onClick={() => void permanentDelete()}><Trash2 size={15} /> Eliminar</Button></> : <Button onClick={() => void changeArchive(true)}><Archive size={15} /> Archivar</Button>}</footer>
  </aside>;
}

export function QuotesHistoryView({ clients, pricing, onOpenProject, onDuplicated }: {
  clients: Client[]; pricing: PricingConfiguration;
  onOpenProject: (projectId: string) => Promise<void>;
  onDuplicated: (workspace: Workspace) => void;
}) {
  const [items, setItems] = useState<QuoteHistoryItem[]>([]);
  const [filters, setFilters] = useState(initialFilters);
  const [detail, setDetail] = useState<QuoteHistoryDetail | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState("");
  const serviceTypes = useMemo(() => [...new Set(items.flatMap((item) => item.serviceTypes.split("|").filter(Boolean)))], [items]);
  const visible = useMemo(() => filterQuoteHistory(items, filters), [items, filters]);

  async function load() {
    setLoading(true); setError("");
    try { setItems(await api.listQuoteHistory()); }
    catch (reason) { setError(String(reason)); }
    finally { setLoading(false); }
  }
  async function openDetail(quoteId: string, revision?: number) {
    setError("");
    try { setDetail(await api.getQuoteHistory(quoteId, revision)); await load(); }
    catch (reason) { setError(String(reason)); }
  }
  useEffect(() => {
    let cancelled = false;
    void api.listQuoteHistory()
      .then((quotes) => { if (!cancelled) setItems(quotes); })
      .catch((reason) => { if (!cancelled) setError(String(reason)); })
      .finally(() => { if (!cancelled) setLoading(false); });
    return () => { cancelled = true; };
  }, []);

  return <div className={`quotes-history ${detail ? "has-detail" : ""}`}>
    <main className="history-main"><header className="market-view__header"><div><span className="eyebrow">Archivo comercial local</span><h1>Cotizaciones</h1><p>Cortes históricos inmutables para consultar, comparar y reutilizar sin recálculos ocultos.</p></div><Button onClick={() => void load()} disabled={loading}><RefreshCw size={16} /> Actualizar</Button></header>
      <div className="history-filters"><label className="search-box"><Search size={16} /><input aria-label="Buscar cotizaciones" placeholder="Proyecto o cliente" value={filters.query} onChange={(event) => setFilters({ ...filters, query: event.target.value })} /></label><Filter size={16} /><Select aria-label="Filtrar por estado" value={filters.status} onChange={(event) => setFilters({ ...filters, status: event.target.value as QuoteHistoryFilters["status"] })}><option value="all">Todos los estados</option>{Object.entries(quoteStatusLabels).map(([value, label]) => <option key={value} value={value}>{label}</option>)}</Select><Select aria-label="Filtrar por servicio" value={filters.serviceType} onChange={(event) => setFilters({ ...filters, serviceType: event.target.value })}><option value="all">Todos los módulos</option>{serviceTypes.map((type) => <option key={type} value={type}>{pricing.pricingEngines.find((engine) => engine.engineKey === type)?.name ?? type}</option>)}</Select><Select aria-label="Filtrar por moneda" value={filters.currency} onChange={(event) => setFilters({ ...filters, currency: event.target.value as QuoteHistoryFilters["currency"] })}><option value="all">ARS y USD</option><option value="ARS">ARS</option><option value="USD">USD</option></Select><Select aria-label="Ordenar cotizaciones" value={filters.sort} onChange={(event) => setFilters({ ...filters, sort: event.target.value as QuoteHistoryFilters["sort"] })}><option value="recent">Más recientes</option><option value="oldest">Más antiguas</option><option value="price-desc">Mayor precio</option><option value="price-asc">Menor precio</option></Select></div>
      {error && <div className="history-error" role="alert"><span>{error}</span><Button onClick={() => void load()}>Reintentar</Button></div>}
      {loading ? <div className="history-loading"><RefreshCw className="spin" size={20} /> Cargando cotizaciones…</div> : items.length === 0 ? <EmptyState eyebrow="Sin cortes históricos" title="Todavía no guardaste una cotización" description="Tus proyectos ya se guardan automáticamente. Desde la calculadora, usá “Guardar cotización” para crear el primer snapshot histórico." /> : visible.length === 0 ? <EmptyState eyebrow="Sin coincidencias" title="No encontramos cotizaciones" description="Probá limpiar la búsqueda o combinar otros filtros." action={<Button onClick={() => setFilters(initialFilters)}>Limpiar filtros</Button>} /> : <section className="quote-ledger" aria-label="Lista de cotizaciones">{visible.map((quote) => <button key={quote.id} className={detail?.quote.id === quote.id ? "is-active" : ""} onClick={() => void openDetail(quote.id)}><span className="quote-ledger__date"><CalendarDays size={15} />{formatDate(quote.savedAt)}</span><span className="quote-ledger__identity"><strong>{quote.projectName}</strong><small>{quote.clientName}</small></span><span className="quote-ledger__services">{quote.serviceTitles || "Sin módulos"}<small>{quote.serviceCount} {quote.serviceCount === 1 ? "módulo" : "módulos"} · Rev. {quote.snapshotRevision}</small></span><span className={`quote-status quote-status--${quote.status}`}>{quoteStatusLabels[quote.status]}</span><span className="quote-ledger__price"><small>{quote.selectedPriceKind === "custom" ? "Personalizado" : "Seleccionado"}</small><strong>{formatMoney(quote.selectedPriceMinor, quote.currency)}</strong></span><ArrowRight size={17} /></button>)}</section>}
    </main>
    {detail && <Detail key={`${detail.quote.id}-${detail.displayedRevision}-${detail.quote.updatedAt}`} detail={detail} clients={clients} onClose={() => { setDetail(null); void load(); }} onReload={openDetail} onOpenProject={onOpenProject} onDuplicated={onDuplicated} />}
  </div>;
}
