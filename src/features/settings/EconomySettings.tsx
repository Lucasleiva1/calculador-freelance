import { useState } from "react";
import { Save } from "lucide-react";
import type { Currency, EconomicProfile, EconomicProfileInput, PricingConfiguration } from "../../domain/types";
import { calculateSustainableRate } from "../../domain/pricingEngine";
import { formatMoney, majorToMinor, minorToInput } from "../../domain/money";
import { Button, Field, Input } from "../../components/ui";

type Form = { income: string; expenses: string; hours: string; reserve: string; margin: string; urgency: string; days: string; vacation: string; manual: string };
const empty: Form = { income: "", expenses: "", hours: "", reserve: "", margin: "", urgency: "", days: "", vacation: "", manual: "" };
const numberOrNull = (value: string) => value.trim() ? Number(value.replace(",", ".")) : null;

function toForm(profile: EconomicProfile | undefined): Form {
  if (!profile) return empty;
  return { income: minorToInput(profile.monthlyIncomeTargetMinor), expenses: minorToInput(profile.monthlyExpensesMinor), hours: profile.billableHoursMicros == null ? "" : String(profile.billableHoursMicros / 1_000_000), reserve: profile.reserveTaxMicros == null ? "" : String(profile.reserveTaxMicros / 10_000), margin: profile.desiredMarginMicros == null ? "" : String(profile.desiredMarginMicros / 10_000), urgency: profile.defaultUrgencyMicros == null ? "" : String(profile.defaultUrgencyMicros / 10_000), days: profile.workDays?.toString() ?? "", vacation: profile.vacationWeeks?.toString() ?? "", manual: minorToInput(profile.manualHourlyRateMinor) };
}

export function EconomySettings({ pricing, onSave }: { pricing: PricingConfiguration; onSave: (input: EconomicProfileInput) => Promise<void> }) {
  const [currency, setCurrency] = useState<Currency>("USD");
  const profile = pricing.economicProfiles.find((item) => item.currency === currency);
  const [form, setForm] = useState<Form>(() => toForm(profile));
  const [status, setStatus] = useState("");
  const selectCurrency = (next: Currency) => { setCurrency(next); setForm(toForm(pricing.economicProfiles.find((item) => item.currency === next))); setStatus(""); };
  const preview: EconomicProfile = { currency, monthlyIncomeTargetMinor: majorToMinor(form.income), monthlyExpensesMinor: majorToMinor(form.expenses), billableHoursMicros: numberOrNull(form.hours) == null ? null : Math.round((numberOrNull(form.hours) ?? 0) * 1_000_000), reserveTaxMicros: numberOrNull(form.reserve) == null ? null : Math.round((numberOrNull(form.reserve) ?? 0) * 10_000), desiredMarginMicros: numberOrNull(form.margin) == null ? null : Math.round((numberOrNull(form.margin) ?? 0) * 10_000), defaultUrgencyMicros: numberOrNull(form.urgency) == null ? null : Math.round((numberOrNull(form.urgency) ?? 0) * 10_000), workDays: numberOrNull(form.days), vacationWeeks: numberOrNull(form.vacation), manualHourlyRateMinor: majorToMinor(form.manual), updatedAt: "" };
  const sustainable = calculateSustainableRate(preview);
  async function submit(event: React.FormEvent) { event.preventDefault(); setStatus("Guardando…"); try { const { updatedAt: _, ...input } = preview; void _; await onSave(input); setStatus("Perfil guardado"); } catch (error) { setStatus(String(error)); } }
  return <form className="settings-form" onSubmit={submit}>
    <section><div className="settings-section-head"><div><span className="eyebrow">Mi economía</span><h2>Tarifa sostenible</h2><p>Calculada únicamente con tus datos. No usa valores de mercado.</p></div><div className="segmented compact"><button type="button" className={currency === "ARS" ? "is-active" : ""} onClick={() => selectCurrency("ARS")}>ARS</button><button type="button" className={currency === "USD" ? "is-active" : ""} onClick={() => selectCurrency("USD")}>USD</button></div></div>
      <div className="form-grid form-grid--3"><Field label={`Ingreso mensual objetivo · ${currency}`}><Input type="number" min="0" step="0.01" value={form.income} onChange={(e) => setForm({ ...form, income: e.target.value })} /></Field><Field label={`Gastos mensuales · ${currency}`}><Input type="number" min="0" step="0.01" value={form.expenses} onChange={(e) => setForm({ ...form, expenses: e.target.value })} /></Field><Field label="Horas facturables / mes"><Input type="number" min="0.01" step="0.01" value={form.hours} onChange={(e) => setForm({ ...form, hours: e.target.value })} /></Field><Field label="Reserva e impuestos · %"><Input type="number" min="0" max="99.99" step="0.01" value={form.reserve} onChange={(e) => setForm({ ...form, reserve: e.target.value })} /></Field><Field label="Margen deseado · %" hint="Margen real: se aplica dividiendo por 1 − margen."><Input type="number" min="0" max="99.99" step="0.01" value={form.margin} onChange={(e) => setForm({ ...form, margin: e.target.value })} /></Field><Field label="Urgencia por defecto · %"><Input type="number" min="0" step="0.01" value={form.urgency} onChange={(e) => setForm({ ...form, urgency: e.target.value })} /></Field><Field label="Días de trabajo / mes"><Input type="number" min="1" step="1" value={form.days} onChange={(e) => setForm({ ...form, days: e.target.value })} /></Field><Field label="Semanas de vacaciones"><Input type="number" min="0" max="51" step="1" value={form.vacation} onChange={(e) => setForm({ ...form, vacation: e.target.value })} /></Field><Field label={`Tarifa manual / hora · ${currency}`} hint="Opcional. Si existe, tiene prioridad sobre la calculada."><Input type="number" min="0" step="0.01" value={form.manual} onChange={(e) => setForm({ ...form, manual: e.target.value })} /></Field></div>
    </section>
    <section className="economy-result"><span className="eyebrow">Referencia interna</span><div><strong>{formatMoney(sustainable.rateMinor, currency)}</strong><span>por hora sostenible</span></div>{sustainable.monthlyRequiredMinor != null && <p>Necesidad mensual bruta: {formatMoney(sustainable.monthlyRequiredMinor, currency)}.</p>}{sustainable.issues.length > 0 && <p>{sustainable.issues.join(" ")}</p>}</section>
    <footer><span>{status}</span><Button variant="accent"><Save size={17} /> Guardar economía</Button></footer>
  </form>;
}
