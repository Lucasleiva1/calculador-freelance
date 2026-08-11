import { useMemo, useState } from "react";
import { ChevronDown, Copy, Plus, RotateCcw, Save, Trash2 } from "lucide-react";
import type { Currency, Preset, QuoteService } from "../../domain/types";
import { applyVideoPreset, type ExternalCost, type VideoConfiguration } from "../../domain/video";
import { majorToMinor, minorToInput } from "../../domain/money";
import { Button, Field, Input, Select } from "../../components/ui";
import { EffortInput } from "../../components/EffortInput";

const pieceTypes = [
  ["reel-short", "Reel / Short"], ["youtube", "YouTube"], ["advertising", "Publicidad"],
  ["institutional", "Institucional"], ["podcast", "Podcast"], ["videoclip", "Videoclip"],
  ["course", "Curso"], ["event", "Evento"], ["other", "Otro"],
];

function ChoiceGroup({ options, value, onChange, multiple = false }: { options: Array<[string, string]>; value: string | string[]; onChange: (value: any) => void; multiple?: boolean }) {
  const selected = Array.isArray(value) ? value : [value];
  return <div className="choice-group">{options.map(([id, label]) => <button type="button" key={id} className={selected.includes(id) ? "is-active" : ""} onClick={() => {
    if (!multiple) return onChange(id);
    onChange(selected.includes(id) ? selected.filter((item) => item !== id) : [...selected, id]);
  }}>{label}</button>)}</div>;
}

export function VideoEditor({
  service,
  config,
  currency,
  presets,
  onChange,
  onSavePreset,
  onUpdatePreset,
  onDeletePreset,
  onRestorePreset,
}: {
  service: QuoteService;
  config: VideoConfiguration;
  currency: Currency;
  presets: Preset[];
  onChange: (config: VideoConfiguration, manualMinor?: number | null, manualReason?: string | null, immediate?: boolean) => void;
  onSavePreset: (name: string, config: VideoConfiguration) => Promise<void>;
  onUpdatePreset: (preset: Preset, config: VideoConfiguration) => Promise<void>;
  onDeletePreset: (preset: Preset) => Promise<void>;
  onRestorePreset: (preset: Preset) => Promise<void>;
}) {
  const [advanced, setAdvanced] = useState(false);
  const [selectedPresetId, setSelectedPresetId] = useState(presets[0]?.id ?? "");
  const [presetName, setPresetName] = useState("");
  const selectedPreset = useMemo(() => presets.find((preset) => preset.id === selectedPresetId), [presets, selectedPresetId]);
  const update = <K extends keyof VideoConfiguration>(key: K, value: VideoConfiguration[K], immediate = false) => onChange({ ...config, [key]: value }, service.manualSubtotalMinor, service.manualReason, immediate);

  function addCost() {
    const cost: ExternalCost = { id: crypto.randomUUID(), name: "", amountMinor: 0, currency, note: "" };
    update("externalCosts", [...config.externalCosts, cost]);
  }

  function patchCost(id: string, patch: Partial<ExternalCost>) {
    update("externalCosts", config.externalCosts.map((cost) => cost.id === id ? { ...cost, ...patch } : cost));
  }

  return <div className="service-editor">
    <section className="preset-strip">
      <div><span className="field__label">Preset de trabajo</span><div className="preset-picker"><Select value={selectedPresetId} onChange={(event) => setSelectedPresetId(event.target.value)}><option value="">Sin preset</option>{presets.map((preset) => <option key={preset.id} value={preset.id}>{preset.name}{preset.origin === "user" ? " · propio" : ""}</option>)}</Select><Button type="button" onClick={() => selectedPreset && onChange(applyVideoPreset(config, selectedPreset.configurationJson), service.manualSubtotalMinor, service.manualReason, true)} disabled={!selectedPreset}>Aplicar</Button></div></div>
      <div className="preset-actions"><Input value={presetName} onChange={(event) => setPresetName(event.target.value)} placeholder="Nombre del nuevo preset" /><Button type="button" variant="ghost" onClick={async () => { if (!presetName.trim()) return; await onSavePreset(presetName.trim(), config); setPresetName(""); }}><Save size={15} /> Guardar como</Button>{selectedPreset && <><Button type="button" variant="ghost" title="Actualizar preset" onClick={() => onUpdatePreset(selectedPreset, config)}><Save size={15} /></Button><Button type="button" variant="ghost" title="Duplicar preset" onClick={() => onSavePreset(`${selectedPreset.name} copia`, config)}><Copy size={15} /></Button>{selectedPreset.origin === "system" ? <Button type="button" variant="ghost" title="Restaurar preset del sistema" onClick={() => onRestorePreset(selectedPreset)}><RotateCcw size={15} /></Button> : <Button type="button" variant="danger" title="Eliminar preset" onClick={() => onDeletePreset(selectedPreset)}><Trash2 size={15} /></Button>}</>}</div>
    </section>

    <div className="editor-section">
      <div className="editor-grid editor-grid--3">
        <Field label="Tipo de pieza" className="span-2"><Select value={config.pieceType} onChange={(event) => update("pieceType", event.target.value, true)}><option value="">Seleccionar</option>{pieceTypes.map(([value, label]) => <option key={value} value={value}>{label}</option>)}</Select></Field>
        <Field label="Cantidad de piezas"><Input type="number" min="1" step="1" value={config.quantity} onChange={(event) => update("quantity", Math.max(1, Number(event.target.value) || 1))} /></Field>
        <Field label="Material bruto" hint="Minutos recibidos"><div className="with-shortcuts"><Input type="number" min="0" value={config.rawMinutes ?? ""} onChange={(event) => update("rawMinutes", event.target.value === "" ? null : Math.max(0, Number(event.target.value)))} /><div>{[5,15,30,60].map((minutes) => <button type="button" key={minutes} onClick={() => update("rawMinutes", minutes, true)}>{minutes}</button>)}</div></div></Field>
        <Field label="Duración final" hint="MM:SS · admite cualquier duración" className="span-2"><div className="with-shortcuts"><Input inputMode="numeric" placeholder="01:30" value={config.finalDuration} onChange={(event) => update("finalDuration", event.target.value)} /><div>{["00:30","01:00","01:30","02:00","03:00","05:00","10:00"].map((duration) => <button type="button" key={duration} onClick={() => update("finalDuration", duration, true)}>{duration}</button>)}</div></div></Field>
        <Field label="Resolución"><Input value="Full HD 1080p" disabled /></Field>
      </div>
    </div>

    <div className="editor-section">
      <div className="editor-grid editor-grid--2">
        <Field label="Nivel de edición"><ChoiceGroup value={config.editingLevel} onChange={(value) => update("editingLevel", value, true)} options={[["basic","Básica"],["professional","Profesional"],["advanced","Avanzada"],["custom","Custom"]]} /></Field>
        <Field label="Revisiones incluidas"><Select value={config.revisions} onChange={(event) => update("revisions", Number(event.target.value), true)}>{[1,2,3,4,5].map((value) => <option key={value}>{value}</option>)}</Select></Field>
        <Field label="Urgencia"><ChoiceGroup value={config.urgency} onChange={(value) => update("urgency", value, true)} options={[["normal","Normal"],["priority","Prioridad"],["48h","48 h"],["24h","24 h"]]} /></Field>
        <Field label={`Importe de urgencia · ${currency}`}><Input type="number" min="0" step="0.01" disabled={config.urgency === "normal"} value={minorToInput(config.urgencyFeeMinor)} onChange={(event) => update("urgencyFeeMinor", majorToMinor(event.target.value) ?? 0)} /></Field>
        <Field label="Formatos" className="span-2"><ChoiceGroup multiple value={config.formats} onChange={(value) => update("formats", value, true)} options={[["16:9","16:9"],["9:16","9:16"],["1:1","1:1"]]} /></Field>
      </div>
    </div>

    <button className={`advanced-toggle ${advanced ? "is-open" : ""}`} type="button" onClick={() => setAdvanced(!advanced)}><Plus size={17} /> Configuración avanzada <ChevronDown size={17} /></button>
    {advanced && <div className="editor-section advanced-panel">
      <div className="editor-grid editor-grid--3">
        <Field label="Color"><Select value={config.color} onChange={(event) => update("color", event.target.value as VideoConfiguration["color"], true)}><option value="none">Sin corrección</option><option value="basic">Corrección básica</option><option value="look">Corrección + look</option></Select></Field>
        <Field label="Audio"><Select value={config.audio} onChange={(event) => update("audio", event.target.value as VideoConfiguration["audio"], true)}><option value="basic">Básico</option><option value="cleanup">Limpieza</option><option value="music-effects">Música + efectos</option><option value="sound-design">Sound design</option></Select></Field>
        <Field label="Subtítulos"><Select value={config.subtitles} onChange={(event) => update("subtitles", event.target.value as VideoConfiguration["subtitles"], true)}><option value="none">No</option><option value="standard">Estándar</option><option value="designed">Diseñados</option></Select></Field>
        <Field label="Video IA"><Select value={config.videoAi} onChange={(event) => update("videoAi", event.target.value as VideoConfiguration["videoAi"], true)}><option value="none">No</option><option value="partial">Generación parcial</option><option value="important">Generación importante</option></Select></Field>
        <Field label="Voz IA"><Select value={String(config.voiceAi)} onChange={(event) => update("voiceAi", event.target.value === "true", true)}><option value="false">No</option><option value="true">Sí</option></Select></Field>
        <Field label="Sonido IA"><Select value={String(config.soundAi)} onChange={(event) => update("soundAi", event.target.value === "true", true)}><option value="false">No</option><option value="true">Sí</option></Select></Field>
        <Field label="Remoción de fondo"><Select value={String(config.backgroundRemoval)} onChange={(event) => update("backgroundRemoval", event.target.value === "true", true)}><option value="false">No</option><option value="true">Sí</option></Select></Field>
        <Field label="Motion"><Select value={config.motion} onChange={(event) => update("motion", event.target.value as VideoConfiguration["motion"], true)}><option value="none">Ninguno</option><option value="basic">Básico</option><option value="ai-assisted">Asistido por IA</option><option value="custom">Custom</option></Select></Field>
        <Field label="Material / B-roll"><Select value={config.broll} onChange={(event) => update("broll", event.target.value as VideoConfiguration["broll"], true)}><option value="client">Cliente entrega todo</option><option value="simple">Búsqueda simple</option><option value="advanced">Búsqueda avanzada</option></Select></Field>
        <Field label="Versiones adicionales"><Input type="number" min="0" step="1" value={config.additionalVersions} onChange={(event) => update("additionalVersions", Math.max(0, Number(event.target.value) || 0))} /></Field>
      </div>
    </div>}

    <div className="editor-section pricing-section">
      <header><div><span className="eyebrow">Base económica</span><h3>Tiempo y costos transparentes</h3></div><p>El precio final y su override se administran en el inspector.</p></header>
      <EffortInput amount={config.effortAmount} unit={config.effortUnit} hoursPerDay={config.hoursPerDay} estimatedHours={config.estimatedHours} onChange={(effort) => onChange({ ...config, effortAmount: effort.amount, effortUnit: effort.unit, hoursPerDay: effort.hoursPerDay, estimatedHours: effort.estimatedHours }, service.manualSubtotalMinor, service.manualReason)} />
      <div className="costs"><div className="costs__header"><span className="field__label">Archivos / costos externos</span><Button type="button" variant="ghost" onClick={addCost}><Plus size={15} /> Añadir costo</Button></div>{config.externalCosts.length === 0 ? <p className="muted-line">No hay costos externos.</p> : config.externalCosts.map((cost) => <div className="cost-row" key={cost.id}><Input aria-label="Nombre del costo" placeholder="Concepto" value={cost.name} onChange={(event) => patchCost(cost.id, { name: event.target.value })} /><Input aria-label="Importe del costo" type="number" min="0" step="0.01" value={minorToInput(cost.amountMinor)} onChange={(event) => patchCost(cost.id, { amountMinor: majorToMinor(event.target.value) ?? 0 })} /><Select aria-label="Moneda del costo" value={cost.currency} onChange={(event) => patchCost(cost.id, { currency: event.target.value as Currency })}><option>USD</option><option>ARS</option></Select><Input aria-label="Nota del costo" placeholder="Nota opcional" value={cost.note} onChange={(event) => patchCost(cost.id, { note: event.target.value })} /><button className="icon-button" aria-label="Eliminar costo" onClick={() => update("externalCosts", config.externalCosts.filter((item) => item.id !== cost.id))}><Trash2 size={16} /></button></div>)}</div>
    </div>
  </div>;
}
