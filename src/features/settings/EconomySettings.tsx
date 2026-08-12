import { useRef, useState } from "react";
import { CircleAlert, CircleCheck, Download, FileText, Save, Upload } from "lucide-react";
import type { Currency, EconomicProfile, EconomicProfileInput, PricingConfiguration, PricingEngine } from "../../domain/types";
import { importNumberInput, parseEconomyImport, type EconomyImportResult } from "../../domain/economyImport";
import { calculateSustainableRate } from "../../domain/pricingEngine";
import { formatMoney, majorToMinor, minorToInput } from "../../domain/money";
import { api } from "../../services/api";
import { Button, EmptyState, Field, Input, Select } from "../../components/ui";

type Form = { income: string; expenses: string; hours: string; reserve: string; margin: string; urgency: string; days: string; vacation: string; manual: string };
const empty: Form = { income: "", expenses: "", hours: "", reserve: "", margin: "", urgency: "", days: "", vacation: "", manual: "" };
const numberOrNull = (value: string) => value.trim() ? Number(value.replace(",", ".")) : null;
const normalizedActivity = (value: string) => value.normalize("NFD").replace(/\p{Diacritic}/gu, "").toLowerCase().replace(/[^a-z0-9]/g, "");

function toForm(profile: EconomicProfile | undefined): Form {
  if (!profile) return empty;
  return { income: minorToInput(profile.monthlyIncomeTargetMinor), expenses: minorToInput(profile.monthlyExpensesMinor), hours: profile.billableHoursMicros == null ? "" : String(profile.billableHoursMicros / 1_000_000), reserve: profile.reserveTaxMicros == null ? "" : String(profile.reserveTaxMicros / 10_000), margin: profile.desiredMarginMicros == null ? "" : String(profile.desiredMarginMicros / 10_000), urgency: profile.defaultUrgencyMicros == null ? "" : String(profile.defaultUrgencyMicros / 10_000), days: profile.workDays?.toString() ?? "", vacation: profile.vacationWeeks?.toString() ?? "", manual: minorToInput(profile.manualHourlyRateMinor) };
}

function readFileAsDataUrl(file: File) {
  return new Promise<string>((resolve, reject) => {
    const reader = new FileReader();
    reader.onload = () => typeof reader.result === "string" ? resolve(reader.result) : reject(new Error("No se pudo leer el archivo."));
    reader.onerror = () => reject(new Error("No se pudo leer el archivo."));
    reader.readAsDataURL(file);
  });
}

function availableEngines(pricing: PricingConfiguration) {
  return pricing.pricingEngines.filter((engine) => engine.status === "active" && engine.calculatorKey !== "unconfigured");
}

function profileFor(pricing: PricingConfiguration, engineId: string, currency: Currency) {
  return pricing.economicProfiles.find((profile) => profile.engineId === engineId && profile.currency === currency);
}

function isConfigured(profile: EconomicProfile | undefined) {
  return profile?.manualHourlyRateMinor != null || calculateSustainableRate(profile ?? null).rateMinor != null;
}

export function EconomySettings({ pricing, onSave, initialCurrency = "ARS", initialEngineKey }: { pricing: PricingConfiguration; onSave: (input: EconomicProfileInput) => Promise<void>; initialCurrency?: Currency; initialEngineKey?: string }) {
  const engines = availableEngines(pricing);
  const initialEngine = engines.find((engine) => engine.engineKey === initialEngineKey) ?? engines[0];
  const [engineId, setEngineId] = useState(initialEngine?.id ?? "");
  const [currency, setCurrency] = useState<Currency>(initialCurrency);
  const profile = profileFor(pricing, engineId, currency);
  const [form, setForm] = useState<Form>(() => toForm(profile));
  const [status, setStatus] = useState("");
  const [imported, setImported] = useState<EconomyImportResult | null>(null);
  const [importBusy, setImportBusy] = useState(false);
  const [exportBusy, setExportBusy] = useState<"ai-guide" | "json-template" | null>(null);
  const [importError, setImportError] = useState("");
  const fileInput = useRef<HTMLInputElement>(null);
  const engine = engines.find((item) => item.id === engineId) ?? null;

  if (!engine) return <EmptyState eyebrow="Economía por profesión" title="No hay profesiones activas" description="Creá o activá un motor para configurar su economía manual." />;
  const selectedEngine: PricingEngine = engine;

  const choose = (nextEngine: PricingEngine, nextCurrency: Currency = currency) => {
    setEngineId(nextEngine.id);
    setCurrency(nextCurrency);
    setForm(toForm(profileFor(pricing, nextEngine.id, nextCurrency)));
    setStatus(""); setImported(null); setImportError("");
  };
  const selectCurrency = (next: Currency) => choose(selectedEngine, next);
  const preview: EconomicProfile = { engineId, currency, monthlyIncomeTargetMinor: majorToMinor(form.income), monthlyExpensesMinor: majorToMinor(form.expenses), billableHoursMicros: numberOrNull(form.hours) == null ? null : Math.round((numberOrNull(form.hours) ?? 0) * 1_000_000), reserveTaxMicros: numberOrNull(form.reserve) == null ? null : Math.round((numberOrNull(form.reserve) ?? 0) * 10_000), desiredMarginMicros: numberOrNull(form.margin) == null ? null : Math.round((numberOrNull(form.margin) ?? 0) * 10_000), defaultUrgencyMicros: numberOrNull(form.urgency) == null ? null : Math.round((numberOrNull(form.urgency) ?? 0) * 10_000), workDays: numberOrNull(form.days), vacationWeeks: numberOrNull(form.vacation), manualHourlyRateMinor: majorToMinor(form.manual), updatedAt: "" };
  const sustainable = calculateSustainableRate(preview);
  const configured = isConfigured(profile);

  async function importDocument(file: File | undefined) {
    if (!file) return;
    if (file.size > 5_000_000) { setImportError("El archivo no puede superar 5 MB."); return; }
    setImportBusy(true); setImportError(""); setImported(null);
    try {
      const isPdf = file.type === "application/pdf" || file.name.toLowerCase().endsWith(".pdf");
      const text = isPdf ? await api.extractEconomyPdfText(await readFileAsDataUrl(file)) : await file.text();
      const result = parseEconomyImport(text);
      if (result.values.activity && normalizedActivity(result.values.activity) !== normalizedActivity(selectedEngine.name)) {
        throw new Error(`Este archivo corresponde a ${result.values.activity}, pero estás configurando ${selectedEngine.name}. Elegí la profesión correcta antes de importarlo.`);
      }
      if (result.values.currency && result.values.currency !== currency) {
        throw new Error(`Este archivo está expresado en ${result.values.currency}, pero el perfil abierto usa ${currency}. Elegí la moneda correcta antes de importarlo.`);
      }
      setImported(result);
      setStatus(result.missingFields.length > 0
        ? `Archivo incompleto para ${selectedEngine.name}: faltan ${result.missingFields.length} campos económicos.`
        : `Archivo completo para ${selectedEngine.name}: se detectaron todos los campos económicos.`);
    } catch (error) { setImportError(error instanceof Error ? error.message : String(error)); setStatus(""); }
    finally { setImportBusy(false); }
  }

  async function saveTemplate(kind: "ai-guide" | "json-template") {
    setExportBusy(kind); setImportError("");
    try {
      const path = await api.saveEconomyTemplate(kind, selectedEngine.name, currency);
      setStatus(path ? `Archivo para ${selectedEngine.name} guardado en: ${path}` : "No se guardó ningún archivo: cerraste el diálogo sin elegir una ubicación.");
    } catch (error) { setImportError(error instanceof Error ? error.message : String(error)); setStatus(""); }
    finally { setExportBusy(null); }
  }

  function applyImportedValues() {
    if (!imported) return;
    const values = imported.values;
    const targetCurrency = values.currency ?? currency;
    const base = targetCurrency === currency ? form : toForm(profileFor(pricing, engineId, targetCurrency));
    setCurrency(targetCurrency);
    setForm({ ...base, income: importNumberInput(values.monthlyIncomeTarget) ?? base.income, expenses: importNumberInput(values.monthlyExpenses) ?? base.expenses, hours: importNumberInput(values.billableHoursPerMonth) ?? base.hours, reserve: importNumberInput(values.reserveTaxPercent) ?? base.reserve, margin: importNumberInput(values.desiredMarginPercent) ?? base.margin, urgency: importNumberInput(values.defaultUrgencyPercent) ?? base.urgency, days: importNumberInput(values.workDaysPerMonth) ?? base.days, vacation: importNumberInput(values.vacationWeeksPerYear) ?? base.vacation, manual: importNumberInput(values.manualHourlyRate) ?? base.manual });
    setStatus(`Datos aplicados solamente a ${selectedEngine.name}. Revisalos y guardá esta economía.`); setImported(null);
  }

  async function submit(event: React.FormEvent) {
    event.preventDefault(); setStatus(`Guardando economía de ${selectedEngine.name}…`);
    try { const { updatedAt: _, ...input } = preview; void _; await onSave(input); setStatus(`Economía de ${selectedEngine.name} guardada.`); }
    catch (error) { setStatus(String(error)); }
  }

  return <form className="settings-form economy-settings" onSubmit={submit}>
    <section className="economy-profession"><div><span className="eyebrow">Economía manual por profesión</span><h2>Estás configurando: {engine.name}</h2><p>Estos datos se usarán únicamente para el precio local/sostenible de esta actividad.</p></div><div className={`economy-profession__status ${configured ? "is-ready" : "is-pending"}`}>{configured ? <CircleCheck size={17} /> : <CircleAlert size={17} />}<span>{configured ? "Configurada" : "Pendiente"}<small>{currency}</small></span></div><div className="economy-profession__controls"><Field label="Profesión / actividad"><Select aria-label="Profesión o actividad" value={engineId} onChange={(event) => choose(engines.find((item) => item.id === event.target.value) ?? engine)}>{engines.map((item) => <option key={item.id} value={item.id}>{item.name} · {isConfigured(profileFor(pricing, item.id, currency)) ? "Configurada" : "Pendiente"}</option>)}</Select></Field><div><span className="field__label">Moneda de este perfil</span><div className="segmented compact"><button type="button" className={currency === "ARS" ? "is-active" : ""} onClick={() => selectCurrency("ARS")}>ARS</button><button type="button" className={currency === "USD" ? "is-active" : ""} onClick={() => selectCurrency("USD")}>USD</button></div></div></div></section>
    <section><div className="settings-section-head"><div><span className="eyebrow">{engine.name}</span><h2>Tarifa sostenible</h2><p>Se calcula únicamente con los datos manuales de esta profesión.</p></div></div>
      <section className="economy-import" aria-labelledby="economy-import-title"><div><span className="eyebrow">Importación para {engine.name}</span><h3 id="economy-import-title">Completá este perfil desde un archivo</h3><p>El archivo se aplicará solamente a <strong>{engine.name}</strong>. Nada se guarda hasta que confirmes “Guardar economía”.</p></div><div className="economy-import__actions"><input ref={fileInput} className="sr-only" type="file" accept=".json,.txt,.md,.markdown,.pdf,application/json,text/plain,text/markdown,application/pdf" onChange={(event) => { void importDocument(event.target.files?.[0]); event.currentTarget.value = ""; }} /><Button type="button" onClick={() => fileInput.current?.click()} disabled={importBusy || exportBusy != null}><Upload size={16} /> {importBusy ? "Leyendo…" : "Importar archivo"}</Button><Button type="button" onClick={() => void saveTemplate("ai-guide")} disabled={importBusy || exportBusy != null}><FileText size={16} /> {exportBusy === "ai-guide" ? "Abriendo…" : "Guardar guía para IA"}</Button><Button type="button" onClick={() => void saveTemplate("json-template")} disabled={importBusy || exportBusy != null}><Download size={16} /> {exportBusy === "json-template" ? "Abriendo…" : "Guardar plantilla JSON"}</Button></div>{importError && <p className="form-error" role="alert">{importError}</p>}{imported && <div className="economy-import__preview" role="status"><strong>{imported.missingFields.length > 0 ? `Archivo incompleto para ${engine.name}` : `Perfil completo detectado para ${engine.name}`}</strong><p>{imported.importedFields.join(" · ")}</p>{imported.missingFields.length > 0 && <div className="form-error"><strong>La IA no completó todos los cuadros.</strong><span>Faltan: {imported.missingFields.join(" · ")}. Volvé a pedirle el JSON completo usando la guía nueva.</span></div>}{imported.warnings.length > 0 && <ul>{imported.warnings.map((warning) => <li key={warning}>{warning}</li>)}</ul>}<div><Button type="button" variant="accent" disabled={imported.missingFields.length > 0} onClick={applyImportedValues}>{imported.missingFields.length > 0 ? "Archivo incompleto" : `Aplicar a ${engine.name}`}</Button><Button type="button" variant="ghost" onClick={() => setImported(null)}>Descartar</Button></div></div>}</section>
      <div className="form-grid form-grid--3"><Field label={`Ingreso mensual objetivo · ${currency}`}><Input type="number" min="0" step="0.01" value={form.income} onChange={(e) => setForm({ ...form, income: e.target.value })} /></Field><Field label={`Gastos mensuales de ${engine.name} · ${currency}`}><Input type="number" min="0" step="0.01" value={form.expenses} onChange={(e) => setForm({ ...form, expenses: e.target.value })} /></Field><Field label="Horas facturables / mes"><Input type="number" min="0.01" step="0.01" value={form.hours} onChange={(e) => setForm({ ...form, hours: e.target.value })} /></Field><Field label="Reserva e impuestos · %"><Input type="number" min="0" max="99.99" step="0.01" value={form.reserve} onChange={(e) => setForm({ ...form, reserve: e.target.value })} /></Field><Field label="Margen deseado · %" hint="Margen real: se aplica dividiendo por 1 − margen."><Input type="number" min="0" max="99.99" step="0.01" value={form.margin} onChange={(e) => setForm({ ...form, margin: e.target.value })} /></Field><Field label="Urgencia por defecto · %"><Input type="number" min="0" step="0.01" value={form.urgency} onChange={(e) => setForm({ ...form, urgency: e.target.value })} /></Field><Field label="Días de trabajo / mes"><Input type="number" min="1" step="1" value={form.days} onChange={(e) => setForm({ ...form, days: e.target.value })} /></Field><Field label="Semanas de vacaciones"><Input type="number" min="0" max="51" step="1" value={form.vacation} onChange={(e) => setForm({ ...form, vacation: e.target.value })} /></Field><Field label={`Tarifa manual de ${engine.name} / hora · ${currency}`} hint="Opcional. Sólo reemplaza la tarifa sostenible de esta profesión."><Input type="number" min="0" step="0.01" value={form.manual} onChange={(e) => setForm({ ...form, manual: e.target.value })} /></Field></div>
    </section>
    <section className="economy-result"><span className="eyebrow">Referencia interna · {engine.name}</span><div><strong>{formatMoney(sustainable.rateMinor, currency)}</strong><span>por hora sostenible</span></div>{sustainable.monthlyRequiredMinor != null && <p>Necesidad mensual bruta: {formatMoney(sustainable.monthlyRequiredMinor, currency)}.</p>}{sustainable.issues.length > 0 && <p>{sustainable.issues.join(" ")}</p>}</section>
    <footer><span>{status}</span><Button variant="accent"><Save size={17} /> Guardar economía de {engine.name}</Button></footer>
  </form>;
}
