import { useState, type FormEvent } from "react";
import { Archive, Check, Info } from "lucide-react";
import type { ProjectResult } from "../../domain/quote";
import { majorToMinor, minorToInput } from "../../domain/money";
import type { SaveQuoteSnapshotInput, Workspace } from "../../domain/types";
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
  const [finalPrice, setFinalPrice] = useState(minorToInput(result.totalMinor ?? workspace.quote.selectedPriceMinor));
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState("");
  const tiers = result.pricingTiers;
  const calculationState = result.totalMinor == null ? "Pendiente" : result.isPartial ? "Parcial" : "Completo";

  async function submit(event: FormEvent) {
    event.preventDefault();
    const selected = majorToMinor(finalPrice);
    if (selected == null) { setError("Elegí uno de los tres precios o ingresá un precio final válido."); return; }
    setSaving(true); setError("");
    try {
      await onSave({
        quoteId: workspace.quote.id, notes, selectedPriceKind: "custom", selectedPriceMinor: selected,
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
        <div className="snapshot-guidance"><Info size={20} /><div><strong>Se guarda el precio que elegiste en la pantalla principal</strong><span>Local/sostenible, Mercado e Internacional permanecen como referencias. Este importe es el que recibe el cliente.</span></div></div>
        <Field label={`Precio final elegido (${workspace.quote.currency})`} hint="Podés hacer un último ajuste sin crear una cuarta categoría de precio."><Input inputMode="decimal" value={finalPrice} onChange={(event) => setFinalPrice(event.target.value)} /></Field>
        {result.unpricedCount > 0 && <p className="quote-save__warning"><Info size={16} />{result.totalMinor == null ? "Completá al menos un módulo o elegí un precio personalizado para guardar el borrador." : "Hay módulos sin precio. Podés guardar el borrador con un importe personalizado; quedará identificado como cálculo parcial."}</p>}
        {error && <p className="form-error" role="alert">{error}</p>}
      </div>
      <div className="modal__actions"><Button type="button" onClick={onClose}>Cancelar</Button><Button type="submit" variant="accent" disabled={saving}><Check size={16} /> {saving ? "Guardando…" : submitLabel ?? defaultSubmitLabel}</Button></div>
    </form>
  </Modal>;
}
