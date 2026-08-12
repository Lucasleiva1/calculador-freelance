import { useEffect, useMemo, useState } from "react";
import { Download, Eye, FileText, LoaderCircle, Save } from "lucide-react";
import { formatMoney } from "../../domain/money";
import type { ClientDocumentConfig, ClientQuoteDocument } from "../../domain/types";
import { api } from "../../services/api";
import { Button, Field, Input, Modal } from "../../components/ui";

type Service = { id: string; title: string; description?: string };

export function ClientDocumentModal({ quoteId, services, snapshotRevision, onClose }: { quoteId: string; services: Service[]; snapshotRevision?: number; onClose: () => void }) {
  const [config, setConfig] = useState<ClientDocumentConfig | null>(null);
  const [document, setDocument] = useState<ClientQuoteDocument | null>(null);
  const [busy, setBusy] = useState<"preview" | "export" | "save" | null>(null);
  const [message, setMessage] = useState("");
  const [error, setError] = useState("");

  useEffect(() => {
    let active = true;
    void api.getClientDocumentConfig(quoteId).then((next) => {
      if (!active) return;
      const defaults = Object.fromEntries(services.filter((service) => service.description).map((service) => [service.id, service.description ?? ""]));
      setConfig({ ...next, serviceDescriptions: { ...defaults, ...next.serviceDescriptions }, snapshotRevision });
    }).catch((reason) => { if (active) setError(String(reason)); });
    return () => { active = false; };
  }, [quoteId, services, snapshotRevision]);

  const publicServices = useMemo(() => [...services].sort((a, b) => a.title.localeCompare(b.title)), [services]);
  const update = (next: Partial<ClientDocumentConfig>) => setConfig((current) => current ? { ...current, ...next } : current);

  async function persist() {
    if (!config) return;
    await api.saveClientDocumentConfig(config);
  }
  async function preview() {
    if (!config) return;
    setBusy("preview"); setError(""); setMessage("");
    try { await persist(); setDocument(await api.createClientQuoteDocument(config)); setMessage("Vista previa actualizada. Sólo muestra los datos públicos."); }
    catch (reason) { setError(String(reason)); }
    finally { setBusy(null); }
  }
  async function exportPdf() {
    if (!config) return;
    setBusy("export"); setError(""); setMessage("");
    try {
      await persist();
      const ready = await api.createClientQuoteDocument(config);
      setDocument(ready);
      const output = await api.exportClientQuotePdf(ready);
      setMessage(`PDF creado en: ${output.path}`);
    } catch (reason) { setError(String(reason)); }
    finally { setBusy(null); }
  }
  async function save() {
    if (!config) return;
    setBusy("save"); setError("");
    try { await persist(); setMessage("Información pública guardada para esta cotización."); }
    catch (reason) { setError(String(reason)); }
    finally { setBusy(null); }
  }

  return <Modal title="Preparar cotización para cliente" onClose={onClose} width="1180px">
    {!config ? <div className="modal__body document-loading"><LoaderCircle className="spin" size={20} /> Cargando datos públicos…</div> : <div className="modal__body client-document-editor">
      <section className="client-document-editor__form">
        <div className="snapshot-guidance"><FileText size={20} /><div><strong>Documento externo, cálculo interno protegido</strong><span>La vista previa y el PDF parten de una versión sanitizada: no incluyen piso, recomendado, premium, costos, fuentes ni notas internas.</span></div></div>
        <fieldset className="document-mode"><legend>Cómo mostrar el precio</legend><label className={config.presentationMode === "itemized" ? "is-active" : ""}><input type="radio" checked={config.presentationMode === "itemized"} onChange={() => update({ presentationMode: "itemized" })} /><span><strong>Desglosado</strong><small>Presenta los servicios y distribuye el precio final elegido.</small></span></label><label className={config.presentationMode === "global" ? "is-active" : ""}><input type="radio" checked={config.presentationMode === "global"} onChange={() => update({ presentationMode: "global" })} /><span><strong>Total único</strong><small>Presenta “Proyecto completo” con un solo importe.</small></span></label></fieldset>
        <Field label="Validez" hint="Es informativa; no cambia la cotización cuando vence."><Input type="date" value={config.validUntil ?? ""} onChange={(event) => update({ validUntil: event.target.value || null })} /></Field>
        <div className="form-grid"><Field label="Alcance público"><textarea className="input textarea" rows={3} value={config.scope ?? ""} onChange={(event) => update({ scope: event.target.value || null })} placeholder="Qué incluye el proyecto." /></Field><Field label="Revisiones incluidas"><Input value={config.revisions ?? ""} onChange={(event) => update({ revisions: event.target.value || null })} placeholder="Ej.: hasta 2 rondas de correcciones" /></Field><Field label="Plazo estimado"><Input value={config.estimatedTimeline ?? ""} onChange={(event) => update({ estimatedTimeline: event.target.value || null })} placeholder="Ej.: 7 días hábiles" /></Field><Field label="Condiciones / notas públicas"><textarea className="input textarea" rows={3} value={config.clientNotes ?? ""} onChange={(event) => update({ clientNotes: event.target.value || null })} placeholder="Solo texto destinado al cliente." /></Field></div>
        {config.presentationMode === "itemized" && <section className="document-descriptions"><span className="eyebrow">Descripción pública de servicios</span>{publicServices.map((service) => <Field key={service.id} label={service.title}><Input value={config.serviceDescriptions[service.id] ?? ""} onChange={(event) => update({ serviceDescriptions: { ...config.serviceDescriptions, [service.id]: event.target.value } })} placeholder="Descripción breve para el cliente (opcional)" /></Field>)}</section>}
        {error && <p className="form-error" role="alert">{error}</p>}
        {message && <p className="document-message" role="status">{message}</p>}
        <div className="modal__actions client-document-editor__actions"><Button onClick={() => void save()} disabled={busy !== null}><Save size={16} /> Guardar detalles</Button><Button onClick={() => void preview()} disabled={busy !== null}><Eye size={16} /> {busy === "preview" ? "Preparando…" : "Previsualizar"}</Button><Button variant="accent" onClick={() => void exportPdf()} disabled={busy !== null}><Download size={16} /> {busy === "export" ? "Exportando…" : "Exportar PDF"}</Button></div>
      </section>
      <ClientDocumentPreview document={document} />
    </div>}
  </Modal>;
}

export function ClientDocumentPreview({ document }: { document: ClientQuoteDocument | null }) {
  if (!document) return <aside className="client-document-preview client-document-preview--empty"><Eye size={24} /><strong>Vista previa</strong><span>Completá los datos públicos y seleccioná “Previsualizar”.</span></aside>;
  return <aside className={`client-document-preview client-document-preview--${document.documentTheme}`} aria-label="Vista previa del documento para cliente">
    <header><div>{document.profile.logoDataUrl && <img src={document.profile.logoDataUrl} alt="Logo profesional" />}<span>COTIZACIÓN</span><h3>{document.profile.businessName || document.profile.displayName || "PRICING OS"}</h3>{document.profile.displayName && document.profile.businessName && <small>{document.profile.displayName}</small>}</div><div><span>NÚMERO</span><strong>{document.quoteNumber}</strong><span>FECHA</span><b>{document.issueDate}</b>{document.validUntil && <><span>VÁLIDA HASTA</span><b>{document.validUntil}</b></>}</div></header>
    <section className="client-document-preview__identity"><div><span>PREPARADO PARA</span><strong>{document.clientName}</strong></div><div><span>PROYECTO</span><strong>{document.projectName}</strong></div></section>
    <section className="client-document-preview__lines">{document.lines.map((line, index) => <article key={`${line.title}-${index}`}><span>{String(index + 1).padStart(2, "0")}</span><div><strong>{line.title}</strong>{line.description && <small>{line.description}</small>}</div><b>{formatMoney(line.priceMinor, document.currency)}</b></article>)}</section>
    <section className="client-document-preview__total"><span>TOTAL FINAL</span><strong>{formatMoney(document.totalMinor, document.currency)}</strong></section>
    {[["ALCANCE", document.scope], ["REVISIONES", document.revisions], ["PLAZO ESTIMADO", document.estimatedTimeline], ["CONDICIONES", document.clientNotes]].filter(([, value]) => value).map(([label, value]) => <section className="client-document-preview__note" key={label}><span>{label}</span><p>{value}</p></section>)}
    <footer>{[document.profile.email, document.profile.phone, document.profile.website, document.profile.location].filter(Boolean).join(" · ")}</footer>
  </aside>;
}
