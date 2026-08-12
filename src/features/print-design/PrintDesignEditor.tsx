import { Clock3, FileCheck2, RefreshCw, Shirt } from "lucide-react";
import { EffortInput } from "../../components/EffortInput";
import type { EffortUnit } from "../../domain/effort";
import type { PricingConfiguration, QuoteService } from "../../domain/types";
import type { ProfessionalServiceConfiguration } from "../../domain/professional";
import {
  canonicalizePrintDesignValues,
  deliveryExtraOptions,
  estimatePrintDesignEffort,
  normalizePrintDesignEffort,
  printDesignSummary,
  printDesignTaskOptions,
  suggestedPrintDesignComplexity,
} from "../../domain/printDesign";
import { Button, Field, Input } from "../../components/ui";

const clientOptions = [
  ["small", "Pequeño / C", "Emprendimiento o cliente individual"],
  ["medium", "Mediano / B", "PyME o marca consolidada"],
  ["large", "Grande / A", "Empresa o institución grande"],
] as const;
const materialOptions = [
  ["ready", "Archivo usable"], ["low-quality", "Baja calidad"], ["screenshot", "Captura"], ["reference-only", "Sólo referencia"],
] as const;
const productOptions = [
  ["shirt", "Remera"], ["hoodie", "Buzo"], ["sock", "Media"], ["other", "Otro"],
] as const;
const toneOptions = [["light", "Claro"], ["dark", "Oscuro"], ["both", "Ambos"]] as const;
const printOptions = [["dtf", "DTF"], ["sublimation", "Sublimación"], ["design-only", "Sólo diseño"]] as const;
const complexityOptions = [["basic", "Básica"], ["intermediate", "Intermedia"], ["complex", "Compleja"]] as const;

function Segmented({ label, value, options, onChange, required = false }: {
  label: string;
  value: unknown;
  options: readonly (readonly [string, string, string?])[];
  onChange: (value: string) => void;
  required?: boolean;
}) {
  return <Field label={`${label}${required ? " *" : ""}`}><div className="print-segments" role="radiogroup" aria-label={label}>{options.map(([key, text, hint]) => <button type="button" role="radio" aria-checked={value === key} className={value === key ? "is-selected" : ""} key={key} onClick={() => onChange(key)}><strong>{text}</strong>{hint && <small>{hint}</small>}</button>)}</div></Field>;
}

function Checklist({ label, values, options, onChange }: {
  label: string;
  values: string[];
  options: readonly (readonly [string, string])[];
  onChange: (values: string[]) => void;
}) {
  return <fieldset className="print-checklist"><legend>{label} *</legend>{options.map(([key, text]) => <label key={key} className={values.includes(key) ? "is-selected" : ""}><input type="checkbox" checked={values.includes(key)} onChange={(event) => onChange(event.target.checked ? [...values, key] : values.filter((item) => item !== key))} /><span>{text}</span></label>)}</fieldset>;
}

export function PrintDesignEditor({ service, clientName, config, pricing, onChange }: {
  service: QuoteService;
  clientName: string;
  config: ProfessionalServiceConfiguration;
  pricing: PricingConfiguration;
  suggestionsEnabled: boolean;
  onChange: (config: ProfessionalServiceConfiguration) => void;
}) {
  const definition = pricing.definitions.find((item) => item.serviceType === service.serviceType);
  const parameters = pricing.parameters.filter((item) => item.serviceDefinitionId === definition?.id && item.enabled);
  const values = normalizePrintDesignEffort(config.parameterValues);
  const tasks = Array.isArray(values.workTasks) ? values.workTasks.filter((item): item is string => typeof item === "string") : [];
  const deliveryExtras = Array.isArray(values.deliveryExtras) ? values.deliveryExtras.filter((item): item is string => typeof item === "string") : [];
  const automaticEstimate = estimatePrintDesignEffort({ ...values, complexityMode: "automatic" });
  const suggestedComplexity = suggestedPrintDesignComplexity(values);
  const effortMode = values.estimatedHoursMode === "manual" ? "manual" : "automatic";
  const effortUnit = ["hours", "days"].includes(String(values.effortUnit)) ? values.effortUnit as EffortUnit : "hours";
  const rawHours = typeof values.estimatedHours === "number" ? values.estimatedHours : Number(values.estimatedHours || 0);
  const effortAmount = typeof values.effortAmount === "number" ? values.effortAmount : null;
  const hoursPerDay = typeof values.hoursPerDay === "number" ? values.hoursPerDay : 8;
  const summary = printDesignSummary(values, parameters, pricing.options);

  function emit(changes: Record<string, unknown>) {
    const next = canonicalizePrintDesignValues({ ...values, ...changes });
    delete next.priceSelection;
    const normalized = normalizePrintDesignEffort(next);
    onChange({ ...config, parameterValues: normalized });
  }

  function setReference(hasReference: boolean) {
    const nextTasks = hasReference ? tasks.filter((task) => task !== "design-from-scratch") : [...new Set([...tasks, "design-from-scratch"] )];
    emit({ hasReference, materialType: hasReference ? values.materialType : undefined, workTasks: nextTasks });
  }

  function useAutomaticTime() {
    emit({ estimatedHours: undefined, effortAmount: undefined, estimatedHoursMode: "automatic" });
  }

  function useAutomaticComplexity() {
    emit({ complexityMode: "automatic", complexity: suggestedComplexity.complexity });
  }

  return <div className="dynamic-editor print-design-editor">
    <section className="editor-section print-design-context">
      <div><span className="eyebrow">Diseño de estampas · formulario v3</span><h2>Cotizar una estampa sin vueltas</h2><p className="muted-line">Definí el alcance real. La app calcula por separado tu precio sostenible, Argentina e internacional.</p></div>
      <div className="print-design-context__facts"><span><b>Trabajo</b>{service.title}</span><span><b>Cliente</b>{clientName}</span></div>
    </section>

    <div className="print-design-steps">
      <section className="print-step"><header><span>1</span><div><h3>Referencia</h3><p>Qué recibís y para qué tipo de cliente trabajás.</p></div></header>
        <Segmented label="¿Existe una referencia?" value={typeof values.hasReference === "boolean" ? String(values.hasReference) : ""} options={[["true", "Sí, tengo referencia"], ["false", "No, hay que crear desde cero"]]} onChange={(value) => setReference(value === "true")} required />
        {values.hasReference === true && <Segmented label="Material recibido" value={values.materialType} options={materialOptions} onChange={(materialType) => emit({ materialType })} required />}
        <Segmented label="Categoría del cliente" value={values.clientTier} options={clientOptions} onChange={(clientTier) => emit({ clientTier })} required />
      </section>

      <section className="print-step"><header><span>2</span><div><h3>Producto</h3><p>Dónde se va a aplicar el diseño.</p></div></header>
        <Segmented label="Producto" value={values.productType} options={productOptions} onChange={(productType) => emit({ productType, otherProduct: productType === "other" ? values.otherProduct : undefined })} required />
        {values.productType === "other" && <Field label="¿Qué producto? *"><Input value={String(values.otherProduct ?? "")} onChange={(event) => emit({ otherProduct: event.target.value })} /></Field>}
        <Segmented label="Tono de la prenda o soporte" value={values.garmentTone} options={toneOptions} onChange={(garmentTone) => emit({ garmentTone })} required />
      </section>

      <section className="print-step"><header><span>3</span><div><h3>Impresión</h3><p>La preparación técnica se cobra sólo cuando corresponde.</p></div></header>
        <Segmented label="Sistema" value={values.printSystem} options={printOptions} onChange={(printSystem) => emit({ printSystem, sublimationFitsA4: printSystem === "sublimation" ? values.sublimationFitsA4 : undefined })} required />
        {values.printSystem === "sublimation" && <Segmented label="¿El diseño entra en una hoja A4?" value={typeof values.sublimationFitsA4 === "boolean" ? String(values.sublimationFitsA4) : ""} options={[["true", "Sí · sin recargo"], ["false", "No · hay que dividirlo (+15%)"]]} onChange={(value) => emit({ sublimationFitsA4: value === "true" })} required />}
        {values.printSystem === "dtf" && <p className="print-step__notice">La preparación DTF suma 15% únicamente sobre las horas núcleo.</p>}
      </section>

      <section className="print-step"><header><span>4</span><div><h3>Trabajo necesario</h3><p>Marcá tareas concretas; no se cobran herramientas ni filtros.</p></div></header>
        <Checklist label="Tareas incluidas" values={tasks} options={printDesignTaskOptions} onChange={(workTasks) => emit({ workTasks })} />
        <div className="complexity-control"><div><span className="eyebrow">Complejidad sugerida</span><strong>{complexityOptions.find(([key]) => key === suggestedComplexity.complexity)?.[1] ?? "Pendiente"}</strong><small>Puntaje automático: {suggestedComplexity.score}{values.complexityMode === "manual" ? ` · aplicada manualmente: ${complexityOptions.find(([key]) => key === values.complexity)?.[1] ?? "pendiente"}` : ""}</small></div>{values.complexityMode === "manual" ? <Button type="button" variant="ghost" onClick={useAutomaticComplexity}><RefreshCw size={15} /> Volver al cálculo automático</Button> : <Button type="button" variant="ghost" onClick={() => emit({ complexityMode: "manual" })}>Cambiar manualmente</Button>}</div>
        {values.complexityMode === "manual" && <Segmented label="Complejidad manual" value={values.complexity} options={complexityOptions} onChange={(complexity) => emit({ complexity, complexityMode: "manual" })} />}
      </section>

      <section className="print-step"><header><span>5</span><div><h3>Tiempo</h3><p>Siempre se guardan horas numéricas, aunque prefieras cargar días.</p></div></header>
        <div className="print-design-time__heading"><Clock3 size={18} /><div><strong>{effortMode === "automatic" ? "Estimación automática" : "Tiempo reemplazado manualmente"}</strong><span>Estimación vigente: {automaticEstimate.hours.toLocaleString("es-AR")} h. El reemplazo manual prevalece hasta restaurarlo.</span></div></div>
        <div className="effort-ranges" aria-label="Atajos de tiempo">{[[0.75, "<1 h"], [1.5, "1–2 h"], [3, "2–4 h"], [6, "4–8 h"], [9, "+8 h"]].map(([hours, label]) => <button type="button" key={label} onClick={() => emit({ effortAmount: hours, effortUnit: "hours", estimatedHours: hours, estimatedHoursMode: "manual" })}>{label}</button>)}</div>
        <EffortInput allowedUnits={["hours", "days"]} amount={effortAmount} unit={effortUnit} hoursPerDay={hoursPerDay} estimatedHours={rawHours || null} onChange={(effort) => emit({ effortAmount: effort.amount, effortUnit: effort.unit, hoursPerDay: effort.hoursPerDay, estimatedHours: effort.estimatedHours, estimatedHoursMode: "manual" })} />
        {effortMode === "manual" && <Button type="button" variant="ghost" onClick={useAutomaticTime}><RefreshCw size={15} /> Usar estimación automática</Button>}
      </section>

      <section className="print-step"><header><span>6</span><div><h3>Entrega</h3><p>PNG, JPG o PDF final está incluido. Sumá sólo editables o variantes.</p></div></header>
        <Checklist label="Entregables adicionales" values={deliveryExtras} options={deliveryExtraOptions} onChange={(next) => emit({ deliveryExtras: next })} />
        <Field label="Observaciones internas" hint="No se muestran automáticamente al cliente."><textarea className="input textarea" rows={3} value={config.notes} onChange={(event) => onChange({ ...config, notes: event.target.value })} /></Field>
      </section>
    </div>

    <section className="print-design-summary" aria-live="polite"><header><FileCheck2 size={20} /><div><span className="eyebrow">Resumen del alcance</span><h3>Lo que se entregará</h3></div></header>{summary.length > 0 ? <ul>{summary.map((item) => <li key={item}>{item}</li>)}</ul> : <div className="print-design-summary__empty"><Shirt size={22} /><span>Completá las seis secciones para construir el resumen.</span></div>}</section>
  </div>;
}
