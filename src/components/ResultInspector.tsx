import { useState } from "react";
import {
  ArrowLeftRight,
  ArrowUpRight,
  BarChart3,
  ChevronDown,
  CircleAlert,
  Clock3,
  Globe2,
  MapPin,
  RefreshCw,
  Settings2,
  ShieldCheck,
  XCircle,
} from "lucide-react";
import type { Currency, MarketObservation, MarketOverview, MarketResearchJob } from "../domain/types";
import type { ProjectResult } from "../domain/quote";
import { parseThreePriceSnapshot, type AutomaticPriceOption } from "../domain/market";
import { convertMinor, formatMoney, formatRate, majorToMinor, minorToInput } from "../domain/money";
import { api } from "../services/api";
import { Button, Field, Input, Modal } from "./ui";

interface ResultInspectorProps {
  result: ProjectResult;
  currency: Currency;
  activeServiceId: string | null;
  suggestionsEnabled: boolean;
  usdToArsMicros?: number | null;
  market: MarketOverview | null;
  marketJob: MarketResearchJob | null;
  onUpdateMarket: (force?: boolean) => Promise<void>;
  onCancelMarket: () => Promise<void>;
  onConfigureEconomy?: () => void;
  onFinalPriceChange?: (finalMinor: number | null, reason: string | null) => void;
}

export function ResultInspector({
  result,
  currency,
  activeServiceId,
  usdToArsMicros = null,
  market,
  marketJob,
  onUpdateMarket,
  onCancelMarket,
  onConfigureEconomy,
  onFinalPriceChange,
}: ResultInspectorProps) {
  const active = result.services.find(({ service }) => service.id === activeServiceId);
  const marketUpdating = marketJob?.status === "RUNNING";
  const snapshot = market?.latestSnapshot ?? null;
  const automatic = parseThreePriceSnapshot(snapshot);
  const fxRate = automatic.fxRateMicros ?? usdToArsMicros;
  const snapshotCurrency = snapshot?.currency ?? currency;
  const localPrice = active?.result.calculatedSubtotalMinor ?? null;
  const localIssues = active?.result.issues ?? [];
  const needsEconomy = localIssues.some((issue) => /economía|tarifa|objetivo mensual|gastos mensuales|horas facturables/iu.test(issue));
  const localPendingDescription = localIssues[0] ?? "Faltan datos del alcance para calcular este precio. Los otros dos precios siguen funcionando.";
  const marketPrice = optionValue(automatic.market, snapshotCurrency, currency, fxRate);
  const [internationalCurrency, setInternationalCurrency] = useState<Currency>(currency);
  const internationalPrice = optionValue(automatic.international, snapshotCurrency, internationalCurrency, fxRate);
  const [breakdownOpen, setBreakdownOpen] = useState(false);
  const [overrideOpen, setOverrideOpen] = useState(Boolean(active?.result.hasOverride));
  const [finalInput, setFinalInput] = useState(() => minorToInput(active?.result.finalSubtotalMinor ?? null));
  const [reason, setReason] = useState(active?.service.manualReason ?? "");
  const [sourcesOpen, setSourcesOpen] = useState(false);

  function choosePrice(value: number | null, sourceCurrency: Currency, label: string) {
    if (value == null || !onFinalPriceChange) return;
    const finalValue = convertMinor(value, sourceCurrency, currency, fxRate);
    if (finalValue == null) return;
    const nextReason = `Elegido desde ${label}`;
    setOverrideOpen(true);
    setFinalInput(minorToInput(finalValue));
    setReason(nextReason);
    onFinalPriceChange(finalValue, nextReason);
  }

  return <aside id="resultado-estimado" className="inspector" tabIndex={-1} aria-label="Resultado del estimado">
    <section className="inspector__primary">
      <span className="eyebrow">Precio final elegido</span>
      <div className="hero-price">
        <small>{result.isPartial ? "Subtotal parcial" : "Total del proyecto"}</small>
        <strong>{formatMoney(result.totalMinor, currency)}</strong>
      </div>
      {result.services.length === 0 && <p className="inspector__message">Agregá un servicio para comenzar la cotización.</p>}

      {active && <>
        <header className="three-prices-heading">
          <div><span className="eyebrow">Tres referencias independientes</span><h2>Elegí según el cliente</h2></div>
          {marketUpdating && <span className="three-prices-updating"><RefreshCw className="spin" size={14} /> Actualizando</span>}
        </header>
        <div className="three-price-grid">
          <PriceCard
            icon={<ShieldCheck size={19} />}
            tone="local"
            title="Local / sostenible"
            eyebrow="Tus parámetros manuales"
            value={localPrice}
            currency={currency}
            description={localPrice == null ? localPendingDescription : "Tu base local calculada sólo con los datos que cargaste."}
            onChoose={localPrice != null && onFinalPriceChange ? () => choosePrice(localPrice, currency, "Precio local / sostenible") : undefined}
            action={localPrice == null && needsEconomy && onConfigureEconomy ? <Button type="button" variant="ghost" onClick={onConfigureEconomy}><Settings2 size={14} /> Completar datos</Button> : undefined}
          />
          <PriceCard
            icon={<MapPin size={19} />}
            tone="market"
            title="Mercado"
            eyebrow="Argentina · automático"
            value={marketPrice}
            currency={currency}
            description={automatic.market ? optionDescription(automatic.market) : "Todavía no hay datos argentinos separados. Actualizá las fuentes."}
            range={optionRange(automatic.market, snapshotCurrency, currency, fxRate)}
            onChoose={marketPrice != null && onFinalPriceChange ? () => choosePrice(marketPrice, currency, "Precio de mercado Argentina") : undefined}
          />
          <PriceCard
            icon={<Globe2 size={19} />}
            tone="international"
            title="Internacional"
            eyebrow={`Automático · mostrado en ${internationalCurrency}`}
            value={internationalPrice}
            currency={internationalCurrency}
            description={automatic.international ? optionDescription(automatic.international) : "Todavía no hay una referencia internacional separada."}
            range={optionRange(automatic.international, snapshotCurrency, internationalCurrency, fxRate)}
            onChoose={internationalPrice != null && onFinalPriceChange ? () => choosePrice(internationalPrice, internationalCurrency, "Precio internacional") : undefined}
            action={<Button type="button" variant="ghost" disabled={!fxRate} onClick={() => setInternationalCurrency((current) => current === "ARS" ? "USD" : "ARS")}><ArrowLeftRight size={14} /> Ver en {internationalCurrency === "ARS" ? "USD" : "ARS"}</Button>}
          />
        </div>

        {automatic.fxRateMicros && <p className="three-prices-fx">Conversión: USD 1 = ARS {formatRate(automatic.fxRateMicros)} · {automatic.fxRateDate ?? "fecha no informada"} · {shortSource(automatic.fxRateSource)}</p>}

        {active.result.issues.length > 0 && localPrice == null && <section className="local-price-requirements" role="status">
          <CircleAlert size={17} /><div><strong>El precio local está pendiente</strong><ul>{active.result.issues.map((issue) => <li key={issue}>{issue}</li>)}</ul></div>
        </section>}

        <label className="override-toggle"><input type="checkbox" checked={overrideOpen} disabled={marketUpdating} onChange={(event) => { setOverrideOpen(event.target.checked); if (!event.target.checked) onFinalPriceChange?.(null, null); }} /> Ajustar el precio final</label>
        {overrideOpen && <div className="override-box"><Field label={`Precio final · ${currency}`}><Input type="number" min="0" step="0.01" value={finalInput} disabled={marketUpdating} onChange={(event) => setFinalInput(event.target.value)} /></Field><Field label="Motivo · opcional"><Input disabled={marketUpdating} value={reason} onChange={(event) => setReason(event.target.value)} /></Field><Button variant="accent" disabled={marketUpdating} onClick={() => onFinalPriceChange?.(majorToMinor(finalInput), reason || null)}>Aplicar precio final</Button></div>}

        <button type="button" className={`breakdown-toggle ${breakdownOpen ? "is-open" : ""}`} aria-expanded={breakdownOpen} onClick={() => setBreakdownOpen(!breakdownOpen)}>Ver desglose local <ChevronDown size={16} aria-hidden="true" /></button>
        {breakdownOpen && <div className="breakdown">{active.result.lines.length === 0 ? <p>El precio local todavía no tiene líneas calculables.</p> : active.result.lines.map((line, index) => <div key={`${line.id ?? line.label}-${index}`}><span>{line.label}<small>{line.detail}</small></span><strong>{line.amountMinor >= 0 ? "+" : "−"}{formatMoney(Math.abs(line.amountMinor), currency)}</strong></div>)}</div>}
      </>}

      <div className="metric"><Clock3 size={19} /><span>Horas estimadas</span><strong>{result.totalHours > 0 ? `${result.totalHours.toLocaleString("es-AR")} h` : "—"}</strong></div>
      <div className="metric"><BarChart3 size={19} /><span>Valor final efectivo / h</span><strong>{formatMoney(result.effectiveHourlyMinor, currency)}</strong></div>
    </section>

    <section className="market-panel three-prices-sources">
      <span className="eyebrow">Fuentes automáticas</span>
      {marketUpdating && marketJob ? <div className="inspector-research"><div><RefreshCw className="spin" size={17} /><strong>Actualizando mercado e internacional</strong><span>{marketJob.completed} / {marketJob.total} fuentes</span></div>{marketJob.items.map((item) => <p key={item.sourceId}><span>{item.sourceName}</span><b>{item.status}</b></p>)}<Button variant="danger" onClick={onCancelMarket}><XCircle size={15} /> Cancelar</Button></div>
        : <>{marketJob?.error && <p className="market-offline">{marketJob.error}</p>}<p>{snapshot ? `Última verificación: ${relative(snapshot.createdAt)}.` : "Todavía no hay datos automáticos."}</p><div className="market-panel__actions"><Button disabled={!active} onClick={() => onUpdateMarket(false)}><RefreshCw size={15} /> Actualizar precios</Button><Button disabled={!market?.observations.length} onClick={() => setSourcesOpen(true)}>Ver fuentes</Button><button className="text-action" onClick={() => { if (window.confirm("¿Forzar una actualización ignorando la caché vigente?")) void onUpdateMarket(true); }}>Forzar actualización</button></div></>}
    </section>
    {sourcesOpen && <MarketEvidenceModal market={market} onClose={() => setSourcesOpen(false)} />}
  </aside>;
}

function PriceCard({ icon, tone, title, eyebrow, value, currency, description, range, onChoose, action }: {
  icon: React.ReactNode;
  tone: "local" | "market" | "international";
  title: string;
  eyebrow: string;
  value: number | null;
  currency: Currency;
  description: string;
  range?: string | null;
  onChoose?: () => void;
  action?: React.ReactNode;
}) {
  return <article className={`price-option-card price-option-card--${tone}`}>
    <header><span>{icon}</span><div><small>{eyebrow}</small><h3>{title}</h3></div></header>
    <strong className="price-option-card__value">{formatMoney(value, currency)}</strong>
    {range && <span className="price-option-card__range">Rango: {range}</span>}
    <p>{description}</p>
    <footer>{onChoose && <Button type="button" variant="accent" onClick={onChoose}>Usar este precio</Button>}{action}</footer>
  </article>;
}

function optionValue(option: AutomaticPriceOption | null, from: Currency, to: Currency, rate: number | null) {
  return option?.suggestedPriceMinor == null ? null : convertMinor(option.suggestedPriceMinor, from, to, rate);
}

function optionRange(option: AutomaticPriceOption | null, from: Currency, to: Currency, rate: number | null) {
  if (!option || option.summary.minimumFilteredMinor == null || option.summary.maximumFilteredMinor == null) return null;
  const minimum = convertMinor(option.summary.minimumFilteredMinor, from, to, rate);
  const maximum = convertMinor(option.summary.maximumFilteredMinor, from, to, rate);
  if (minimum == null || maximum == null) return null;
  return `${formatMoney(minimum, to)} — ${formatMoney(maximum, to)}`;
}

function optionDescription(option: AutomaticPriceOption) {
  const confidence = option.summary.confidenceLevel === "HIGH" ? "alta" : option.summary.confidenceLevel === "MEDIUM" ? "media" : "inicial";
  return `${option.summary.comparableCount} referencias comparables · ${option.summary.sourceCount} fuentes · confianza ${confidence}.`;
}

function shortSource(value: string | null) {
  if (!value) return "fuente no informada";
  try { return new URL(value).hostname.replace(/^www\./, ""); } catch { return value; }
}

function MarketEvidenceModal({ market, onClose }: { market: MarketOverview | null; onClose: () => void }) {
  return <Modal title="Fuentes utilizadas" onClose={onClose} width="1080px"><div className="modal__body observations-modal">{!market?.observations.length ? <p>No hay observaciones en este snapshot.</p> : <div className="observation-table" role="table" aria-label="Evidencia utilizada en el snapshot" tabIndex={0}><div className="observation-row observation-row--head" role="row"><span role="columnheader">Fuente</span><span role="columnheader">Servicio</span><span role="columnheader">Precio</span><span role="columnheader">Unidad</span><span role="columnheader">Fecha</span><span role="columnheader">Uso</span></div>{market.observations.map((row) => <ObservationRow key={row.id} row={row} />)}</div>}<div className="modal__actions"><Button onClick={onClose}>Cerrar</Button></div></div></Modal>;
}

function ObservationRow({ row }: { row: MarketObservation }) {
  const included = row.snapshotIncluded ?? row.comparisonEligibility === "ELIGIBLE";
  const reason = row.snapshotExclusionReason || row.exclusionReason;
  return <div className="observation-row" role="row"><span role="cell"><strong>{row.sourceName}</strong><small>{row.sourceType.replaceAll("_", " ")}</small></span><span role="cell">{row.subservice || row.serviceType}<small>{row.region}</small></span><span role="cell">{observationPrice(row)}{row.convertedValueMinor != null && row.convertedCurrency && <small>Convertido: {money(row.convertedValueMinor, row.convertedCurrency)} · tasa {row.exchangeRateMicros == null ? "—" : (row.exchangeRateMicros / 10_000).toLocaleString("es-AR")} · {row.exchangeRateDate || "sin fecha"}</small>}</span><span role="cell">{row.priceType}<small>{row.unit}</small></span><span role="cell">{new Date(row.publishedAt || row.retrievedAt).toLocaleDateString("es-AR")}</span><span role="cell"><b>{included ? "Sí" : "No"}</b><small>{reason}</small><button type="button" className="source-link-button" onClick={() => void api.openMarketSource(row.sourceUrl)}>Original <ArrowUpRight size={13} aria-hidden="true" /></button></span></div>;
}

function observationPrice(row: MarketObservation) {
  if (row.priceValueMinor != null) return money(row.priceValueMinor, row.currency);
  const minimum = row.priceMinMinor == null ? "—" : money(row.priceMinMinor, row.currency);
  const maximum = row.priceMaxMinor == null ? "—" : money(row.priceMaxMinor, row.currency);
  return `${minimum} — ${maximum}`;
}

function money(value: number, currency: string) {
  try { return new Intl.NumberFormat("es-AR", { style: "currency", currency, maximumFractionDigits: 2 }).format(value / 100); }
  catch { return `${currency} ${(value / 100).toLocaleString("es-AR")}`; }
}

function relative(raw: string) {
  const hours = Math.max(0, Math.floor((Date.now() - new Date(raw).getTime()) / 3_600_000));
  return hours < 1 ? "hace menos de 1 h" : hours < 48 ? `hace ${hours} h` : `el ${new Date(raw).toLocaleDateString("es-AR")}`;
}
