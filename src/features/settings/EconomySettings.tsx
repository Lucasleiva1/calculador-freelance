import { useRef, useState } from "react";
import { Download, FileText, Save, Upload } from "lucide-react";
import type { Currency, EconomicProfile, EconomicProfileInput, PricingConfiguration } from "../../domain/types";
import { importNumberInput, parseEconomyImport, type EconomyImportResult } from "../../domain/economyImport";
import { calculateSustainableRate } from "../../domain/pricingEngine";
import { formatMoney, majorToMinor, minorToInput } from "../../domain/money";
import { api } from "../../services/api";
import { Button, Field, Input } from "../../components/ui";

type Form = { income: string; expenses: string; hours: string; reserve: string; margin: string; urgency: string; days: string; vacation: string; manual: string };
const empty: Form = { income: "", expenses: "", hours: "", reserve: "", margin: "", urgency: "", days: "", vacation: "", manual: "" };
const numberOrNull = (value: string) => value.trim() ? Number(value.replace(",", ".")) : null;

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

export function EconomySettings({ pricing, onSave, initialCurrency = "USD" }: { pricing: PricingConfiguration; onSave: (input: EconomicProfileInput) => Promise<void>; initialCurrency?: Currency }) {
  const [currency, setCurrency] = useState<Currency>(initialCurrency);
  const profile = pricing.economicProfiles.find((item) => item.currency === currency);
  const [form, setForm] = useState<Form>(() => toForm(profile));
  const [status, setStatus] = useState("");
  const [imported, setImported] = useState<EconomyImportResult | null>(null);
  const [importBusy, setImportBusy] = useState(false);
  const [importError, setImportError] = useState("");
  const fileInput = useRef<HTMLInputElement>(null);
  const templateHref = `${import.meta.env.BASE_URL}templates/economia-para-importar.json`;
  const guideHref = `${import.meta.env.BASE_URL}templates/prompt-para-chatgpt-mi-economia.md`;

  const selectCurrency = (next: Currency) => { setCurrency(next); setForm(toForm(pricing.economicProfiles.find((item) => item.currency === next))); setStatus(""); };
  const preview: EconomicProfile = { currency, monthlyIncomeTargetMinor: majorToMinor(form.income), monthlyExpensesMinor: majorToMinor(form.expenses), billableHoursMicros: numberOrNull(form.hours) == null ? null : Math.round((numberOrNull(form.hours) ?? 0) * 1_000_000), reserveTaxMicros: numberOrNull(form.reserve) == null ? null : Math.round((numberOrNull(form.reserve) ?? 0) * 10_000), desiredMarginMicros: numberOrNull(form.margin) == null ? null : Math.round((numberOrNull(form.margin) ?? 0) * 10_000), defaultUrgencyMicros: numberOrNull(form.urgency) == null ? null : Math.round((numberOrNull(form.urgency) ?? 0) * 10_000), workDays: numberOrNull(form.days), vacationWeeks: numberOrNull(form.vacation), manualHourlyRateMinor: majorToMinor(form.manual), updatedAt: "" };
  const sustainable = calculateSustainableRate(preview);

  async function importDocument(file: File | undefined) {
    if (!file) return;
    if (file.size > 5_000_000) { setImportError("El archivo no puede superar 5 MB."); return; }
    setImportBusy(true); setImportError(""); setImported(null);
    try {
      const isPdf = file.type === "application/pdf" || file.name.toLowerCase().endsWith(".pdf");
      const text = isPdf ? await api.extractEconomyPdfText(await readFileAsDataUrl(file)) : await file.text();
      const result = parseEconomyImport(text);
      setImported(result);
      setStatus(`Archivo leído: se detectaron ${result.importedFields.length} ${result.importedFields.length === 1 ? "dato" : "datos"}. Revisalos antes de aplicarlos.`);
    } catch (error) {
      setImportError(error instanceof Error ? error.message : String(error));
      setStatus("");
    } finally { setImportBusy(false); }
  }

  function applyImportedValues() {
    if (!imported) return;
    const values = imported.values;
    const targetCurrency = values.currency ?? currency;
    const base = targetCurrency === currency ? form : toForm(pricing.economicProfiles.find((item) => item.currency === targetCurrency));
    setCurrency(targetCurrency);
    setForm({
      ...base,
      income: importNumberInput(values.monthlyIncomeTarget) ?? base.income,
      expenses: importNumberInput(values.monthlyExpenses) ?? base.expenses,
      hours: importNumberInput(values.billableHoursPerMonth) ?? base.hours,
      reserve: importNumberInput(values.reserveTaxPercent) ?? base.reserve,
      margin: importNumberInput(values.desiredMarginPercent) ?? base.margin,
      urgency: importNumberInput(values.defaultUrgencyPercent) ?? base.urgency,
      days: importNumberInput(values.workDaysPerMonth) ?? base.days,
      vacation: importNumberInput(values.vacationWeeksPerYear) ?? base.vacation,
      manual: importNumberInput(values.manualHourlyRate) ?? base.manual,
    });
    setStatus("Datos aplicados al formulario. Revisalos y presioná Guardar economía para confirmarlos.");
    setImported(null);
  }

  async function submit(event: React.FormEvent) {
    event.preventDefault(); setStatus("Guardando…");
    try { const { updatedAt: _, ...input } = preview; void _; await onSave(input); setStatus("Perfil guardado"); }
    catch (error) { setStatus(String(error)); }
  }

  return <form className="settings-form" onSubmit={submit}>
    <section><div className="settings-section-head"><div><span className="eyebrow">Mi economía</span><h2>Tarifa sostenible</h2><p>Calculada únicamente con tus datos. No usa valores de mercado.</p></div><div className="segmented compact"><button type="button" className={currency === "ARS" ? "is-active" : ""} onClick={() => selectCurrency("ARS")}>ARS</button><button type="button" className={currency === "USD" ? "is-active" : ""} onClick={() => selectCurrency("USD")}>USD</button></div></div>
      <section className="economy-import" aria-labelledby="economy-import-title"><div><span className="eyebrow">Importación local</span><h3 id="economy-import-title">Completá el formulario desde un archivo</h3><p>Importá JSON, TXT, MD o un PDF con texto. Nada se guarda hasta que revises los campos y confirmes “Guardar economía”.</p></div><div className="economy-import__actions"><input ref={fileInput} className="sr-only" type="file" accept=".json,.txt,.md,.markdown,.pdf,application/json,text/plain,text/markdown,application/pdf" onChange={(event) => { void importDocument(event.target.files?.[0]); event.currentTarget.value = ""; }} /><Button type="button" onClick={() => fileInput.current?.click()} disabled={importBusy}><Upload size={16} /> {importBusy ? "Leyendo…" : "Importar archivo"}</Button><a href={guideHref} download="guia-para-chatgpt-mi-economia.md"><FileText size={16} /> Descargar guía para IA</a><a href={templateHref} download="economia-para-importar.json"><Download size={16} /> Descargar plantilla JSON</a></div>{importError && <p className="form-error" role="alert">{importError}</p>}{imported && <div className="economy-import__preview" role="status"><strong>Datos detectados</strong><p>{imported.importedFields.join(" · ")}</p>{imported.warnings.length > 0 && <ul>{imported.warnings.map((warning) => <li key={warning}>{warning}</li>)}</ul>}<div><Button type="button" variant="accent" onClick={applyImportedValues}>Aplicar al formulario</Button><Button type="button" variant="ghost" onClick={() => setImported(null)}>Descartar</Button></div></div>}</section>
      <div className="form-grid form-grid--3"><Field label={`Ingreso mensual objetivo · ${currency}`}><Input type="number" min="0" step="0.01" value={form.income} onChange={(e) => setForm({ ...form, income: e.target.value })} /></Field><Field label={`Gastos mensuales · ${currency}`}><Input type="number" min="0" step="0.01" value={form.expenses} onChange={(e) => setForm({ ...form, expenses: e.target.value })} /></Field><Field label="Horas facturables / mes"><Input type="number" min="0.01" step="0.01" value={form.hours} onChange={(e) => setForm({ ...form, hours: e.target.value })} /></Field><Field label="Reserva e impuestos · %"><Input type="number" min="0" max="99.99" step="0.01" value={form.reserve} onChange={(e) => setForm({ ...form, reserve: e.target.value })} /></Field><Field label="Margen deseado · %" hint="Margen real: se aplica dividiendo por 1 − margen."><Input type="number" min="0" max="99.99" step="0.01" value={form.margin} onChange={(e) => setForm({ ...form, margin: e.target.value })} /></Field><Field label="Urgencia por defecto · %"><Input type="number" min="0" step="0.01" value={form.urgency} onChange={(e) => setForm({ ...form, urgency: e.target.value })} /></Field><Field label="Días de trabajo / mes"><Input type="number" min="1" step="1" value={form.days} onChange={(e) => setForm({ ...form, days: e.target.value })} /></Field><Field label="Semanas de vacaciones"><Input type="number" min="0" max="51" step="1" value={form.vacation} onChange={(e) => setForm({ ...form, vacation: e.target.value })} /></Field><Field label={`Tarifa manual / hora · ${currency}`} hint="Opcional. Si existe, tiene prioridad sobre la calculada."><Input type="number" min="0" step="0.01" value={form.manual} onChange={(e) => setForm({ ...form, manual: e.target.value })} /></Field></div>
    </section>
    <section className="economy-result"><span className="eyebrow">Referencia interna</span><div><strong>{formatMoney(sustainable.rateMinor, currency)}</strong><span>por hora sostenible</span></div>{sustainable.monthlyRequiredMinor != null && <p>Necesidad mensual bruta: {formatMoney(sustainable.monthlyRequiredMinor, currency)}.</p>}{sustainable.issues.length > 0 && <p>{sustainable.issues.join(" ")}</p>}</section>
    <footer><span>{status}</span><Button variant="accent"><Save size={17} /> Guardar economía</Button></footer>
  </form>;
}
