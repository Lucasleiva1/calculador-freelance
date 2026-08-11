import { useState, type FormEvent } from "react";
import { Archive, Check, Info } from "lucide-react";
import type { ProjectResult } from "../../domain/quote";
import { formatMoney, majorToMinor, minorToInput } from "../../domain/money";
import type { QuotePriceKind, SaveQuoteSnapshotInput, Workspace } from "../../domain/types";
import { Button, Field, Input, Modal } from "../../components/ui";

export function SaveQuoteModal({ workspace, result, onClose, onSave, title, submitLabel }: {
  workspace: Workspace;
  result: ProjectResult;
  onClose: () => void;
  onSave: (input: SaveQuoteSnapshotInput) => Promise<void>;
  title?: string;
  submitLabel?: string;
}) {
  const [notes, setNotes] = useState(workspace.quote.notes ?? "");
  const [kind, setKind] = useState<QuotePriceKind>(workspace.quote.selectedPriceKind ?? "recommended");
  const [custom, setCustom] = useState(minorToInput(workspace.quote.selectedPriceKind === "custom" ? workspace.quote.selectedPriceMinor : null));
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState("");
  const tiers = result.pricingTiers;
  const calculationState = result.totalMinor == null ? "Pendiente" : result.isPartial ? "Parcial" : "Completo";
  const options: Array<{ id: QuotePriceKind; label: string; value: number | null; description: string }> = [
    { id: "floor", label: "Piso", value: tiers.floorMinor, description: "Mínimo sostenible según los costos y reglas guardados." },
    { id: "recommended", label: "Recomendado", value: tiers.recommendedMinor, description: "Objetivo comercial sugerido por los motores actuales." },
    { id: "premium", label: "Premium", value: tiers.premiumMinor, description: "Valor superior para más margen o posicionamiento." },
    { id: "custom", label: "Personalizado", value: majorToMinor(custom), description: "Un importe elegido por vos; no modifica los cálculos de referencia." },
  ];

  async function submit(event: FormEvent) {
    event.preventDefault();
    const selected = options.find((option) => option.id === kind)?.value ?? null;
    if (selected == null) { setError(kind === "custom" ? "Ingresá un precio personalizado válido." : "Ese nivel todavía no tiene un importe calculado."); return; }
    setSaving(true); setError("");
    try {
      await onSave({
        quoteId: workspace.quote.id, notes, selectedPriceKind: kind, selectedPriceMinor: selected,
        floorTotalMinor: tiers.floorMinor, recommendedTotalMinor: tiers.recommendedMinor,
        premiumTotalMinor: tiers.premiumMinor, totalHoursMicros: Math.round(result.totalHours * 1_000_000),
        externalCostsMinor: result.externalCostsMinor, effectiveHourlyMinor: result.effectiveHourlyMinor,
        marginMicros: result.marginMicros, reason: workspace.quote.snapshotRevision > 0 ? "calculation_update" : "manual_save",
      });
      onClose();
    } catch (reason) { setError(String(reason)); }
    finally { setSaving(false); }
  }

  const defaultSubmitLabel = workspace.quote.snapshotRevision > 0 ? "Crear revisión" : "Guardar cotización";
  return <Modal title={title ?? (workspace.quote.snapshotRevision > 0 ? "Guardar nueva revisión" : "Guardar cotización")} onClose={onClose} width="720px">
    <form onSubmit={submit}>
      <div className="modal__body quote-save">
        <div className="snapshot-guidance"><Archive size={20} /><div><strong>El proyecto ya se guarda automáticamente</strong><span>Esta acción crea un corte histórico: precios, módulos, parámetros y fuentes quedan congelados en una revisión.</span></div></div>
        <div className="quote-save__context"><div><span>Proyecto</span><strong>{workspace.project.name}</strong></div><div><span>Cliente</span><strong>{workspace.project.clientName}</strong></div><div><span>Estado del cálculo</span><strong>{calculationState}</strong></div></div>
        <Field label="Notas de la cotización" hint="Guardá alcance, condiciones o aclaraciones que necesites recordar al reutilizar este proyecto.">
          <textarea className="input textarea" rows={3} value={notes} onChange={(event) => setNotes(event.target.value)} placeholder="Ej.: incluye dos rondas de cambios y entrega master 16:9" />
        </Field>
        <fieldset className="quote-price-picker"><legend>Precio que querés dejar seleccionado</legend>{options.map((option) => { const unavailable = option.id !== "custom" && option.value == null; return <label key={option.id} className={`${kind === option.id ? "is-active" : ""}${unavailable ? " is-unavailable" : ""}`}><input type="radio" name="quote-price" checked={kind === option.id} disabled={unavailable} onChange={() => setKind(option.id)} /><span><strong>{option.label}</strong><small>{unavailable ? "Todavía no hay un importe completo para este nivel." : option.description}</small></span><b>{option.id === "custom" && kind === "custom" ? "Definilo abajo" : formatMoney(option.value, workspace.quote.currency)}</b></label>; })}</fieldset>
        {kind === "custom" && <Field label={`Precio personalizado (${workspace.quote.currency})`} hint="Se conserva junto a piso, recomendado y premium para que después puedas comparar tu decisión."><Input inputMode="decimal" value={custom} onChange={(event) => setCustom(event.target.value)} autoFocus /></Field>}
        {result.unpricedCount > 0 && <p className="quote-save__warning"><Info size={16} />{result.totalMinor == null ? "Completá al menos un módulo o elegí un precio personalizado para guardar el borrador." : "Hay módulos sin precio. Podés guardar el borrador con un importe personalizado; quedará identificado como cálculo parcial."}</p>}
        {error && <p className="form-error" role="alert">{error}</p>}
      </div>
      <div className="modal__actions"><Button type="button" onClick={onClose}>Cancelar</Button><Button type="submit" variant="accent" disabled={saving}><Check size={16} /> {saving ? "Guardando…" : submitLabel ?? defaultSubmitLabel}</Button></div>
    </form>
  </Modal>;
}
