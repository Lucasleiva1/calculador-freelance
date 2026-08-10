import { useMemo, useState } from "react";
import type { Client, CreateProjectInput, Currency, MarketScope } from "../domain/types";
import { Button, Field, Input, Modal, Select } from "./ui";

export function NewProjectModal({ clients, onClose, onCreate }: { clients: Client[]; onClose: () => void; onCreate: (input: CreateProjectInput) => Promise<void> }) {
  const activeClients = useMemo(() => clients.filter((client) => client.status === "active"), [clients]);
  const [name, setName] = useState("");
  const [clientMode, setClientMode] = useState<"existing" | "new">(activeClients.length ? "existing" : "new");
  const [clientId, setClientId] = useState(activeClients[0]?.id ?? "");
  const [clientName, setClientName] = useState("");
  const [company, setCompany] = useState("");
  const [currency, setCurrency] = useState<Currency>("USD");
  const [marketScope, setMarketScope] = useState<MarketScope>("argentina");
  const [error, setError] = useState("");
  const [submitting, setSubmitting] = useState(false);

  async function submit(event: React.FormEvent) {
    event.preventDefault();
    if (!name.trim()) return setError("Ingresá el nombre del proyecto.");
    if (clientMode === "existing" && !clientId) return setError("Seleccioná un cliente.");
    if (clientMode === "new" && !clientName.trim()) return setError("Ingresá el nombre del cliente.");
    setSubmitting(true); setError("");
    try {
      await onCreate({
        name: name.trim(),
        clientId: clientMode === "existing" ? clientId : undefined,
        newClient: clientMode === "new" ? { name: clientName.trim(), company: company.trim() || undefined } : undefined,
        currency,
        marketScope,
      });
      onClose();
    } catch (reason) {
      setError(String(reason));
    } finally { setSubmitting(false); }
  }

  return <Modal title="Nuevo proyecto" onClose={onClose}>
    <form onSubmit={submit} className="modal__body form-stack">
      <Field label="Nombre del proyecto"><Input autoFocus value={name} onChange={(event) => setName(event.target.value)} placeholder="Ej. Campaña Agosto 2026" /></Field>
      <fieldset className="segmented-field"><legend>Cliente</legend><div className="segmented"><button type="button" className={clientMode === "existing" ? "is-active" : ""} onClick={() => setClientMode("existing")} disabled={!activeClients.length}>Cliente existente</button><button type="button" className={clientMode === "new" ? "is-active" : ""} onClick={() => setClientMode("new")}>Crear cliente</button></div></fieldset>
      {clientMode === "existing" ? <Field label="Seleccionar cliente"><Select value={clientId} onChange={(event) => setClientId(event.target.value)}>{activeClients.map((client) => <option key={client.id} value={client.id}>{client.name}{client.company ? ` · ${client.company}` : ""}</option>)}</Select></Field> : <div className="form-grid"><Field label="Nombre del cliente"><Input value={clientName} onChange={(event) => setClientName(event.target.value)} /></Field><Field label="Empresa (opcional)"><Input value={company} onChange={(event) => setCompany(event.target.value)} /></Field></div>}
      <div className="form-grid"><Field label="Moneda"><Select value={currency} onChange={(event) => setCurrency(event.target.value as Currency)}><option>USD</option><option>ARS</option></Select></Field><Field label="Mercado de referencia"><Select value={marketScope} onChange={(event) => setMarketScope(event.target.value as MarketScope)}><option value="argentina">Argentina</option><option value="international">Internacional</option><option value="both">Ambos</option></Select></Field></div>
      {error && <div className="form-error" role="alert">{error}</div>}
      <footer className="modal__actions"><Button type="button" variant="ghost" onClick={onClose}>Cancelar</Button><Button type="submit" variant="accent" disabled={submitting}>{submitting ? "Creando…" : "Crear proyecto"}</Button></footer>
    </form>
  </Modal>;
}

