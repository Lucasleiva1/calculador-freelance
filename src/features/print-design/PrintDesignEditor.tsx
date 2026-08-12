import { ChevronDown, Clock3, FileCheck2, RefreshCw, Shirt } from "lucide-react";
import { DynamicFields } from "../../components/DynamicFields";
import { EffortInput } from "../../components/EffortInput";
import type { EffortUnit } from "../../domain/effort";
import type { PricingConfiguration, QuoteService } from "../../domain/types";
import type { ProfessionalServiceConfiguration } from "../../domain/professional";
import { estimatePrintDesignEffort, normalizePrintDesignEffort, printDesignSummary, redundantPrintDesignOperations } from "../../domain/printDesign";
import { Button, Field } from "../../components/ui";

const groups = [
  { title: "Datos y tipo de trabajo", description: "Identificá el pedido y las intervenciones solicitadas.", keys: ["clientType", "briefDescription", "mainWorkType", "additionalOperations"] },
  { title: "Complejidad y material recibido", description: "Evaluá el alcance general y el estado real del archivo de entrada.", keys: ["complexity", "inputQuality", "inputConditions"] },
  { title: "Recorte, limpieza y restauración", description: "Detallá el trabajo necesario antes de diseñar o imprimir.", keys: ["backgroundLevel", "backgroundDetails", "restorationLevel", "restorationTasks"] },
  { title: "Vectorización y composición", description: "Registrá el redibujo, los elementos y el armado visual.", keys: ["vectorizationLevel", "vectorizationElementCount", "vectorizationFeatures", "deliverEditableVector", "compositionLevel", "compositionElementCount", "compositionFeatures"] },
  { title: "IA, tipografía y color", description: "Indicá las herramientas y tratamientos creativos del encargo.", keys: ["aiLevel", "aiActions", "aiGenerationCount", "typographyLevel", "typographyActions", "colorLevel", "colorActions"] },
  { title: "Preparación para estampado", description: "Configurá DTF, sublimación, medidas, resolución y archivos finales.", keys: ["printOutput", "printActions", "finalWidth", "finalHeight", "dimensionUnit", "finalResolution", "deliveryFormat", "halftoneLevel", "halftoneDetails"] },
  { title: "Recursos y cantidad de elementos", description: "Dimensioná la búsqueda y la cantidad de material a trabajar.", keys: ["resourceSearchLevel", "resourceTypes", "resourceCount", "elementCountBand"] },
  { title: "Propuestas, revisiones y variantes", description: "Dejá claro qué alternativas y correcciones incluye el alcance.", keys: ["initialProposals", "includedRevisions", "hasExtraRevisions", "extraRevisionValue", "variantLevel", "variantTypes"] },
  { title: "Tamaños, ubicaciones y editable", description: "Definí entregables, aplicaciones sobre la prenda y archivos fuente.", keys: ["printLocations", "finalSizeCount", "editableDelivery"] },
  { title: "Origen, urgencia y tiempo", description: "Cerrá el alcance con el punto de partida, la prioridad y la estimación.", keys: ["designOrigin", "urgency"] },
] as const;

const complexityHelp = [
  ["Básico", "Poca intervención manual y un archivo relativamente limpio."],
  ["Medio", "Retoque visible, limpieza o vectorización con correcciones."],
  ["Alto", "Composición, reconstrucción o tratamiento técnico importante."],
  ["Premium", "Creación compleja desde cero, varios recursos o IA con retoque avanzado."],
] as const;

const groupTriggers: Record<string, { main?: string[]; operations?: string[]; values?: string[] }> = {
  "Recorte, limpieza y restauración": { main: ["background-simple", "background-complex", "image-cleaning", "restore-image"], operations: ["remove-background", "clean-edges", "reconstruct-missing", "fix-resolution"] },
  "Vectorización y composición": { main: ["vector-basic", "vector-corrected", "redraw-partial", "redraw-full", "composition-simple", "composition-medium", "composition-complex", "design-from-scratch", "adapt-existing"], operations: ["vectorize", "redraw", "compose"], values: ["vectorizationLevel", "compositionLevel"] },
  "IA, tipografía y color": { main: ["ai-generation", "ai-image-edit", "composition-simple", "composition-medium", "composition-complex", "design-from-scratch"], operations: ["generate-ai", "improve-ai", "inpainting", "add-text", "recreate-text", "find-similar-font", "adjust-colors", "change-colors", "black-white"], values: ["aiLevel", "typographyLevel", "colorLevel"] },
  "Recursos y cantidad de elementos": { main: ["composition-medium", "composition-complex", "design-from-scratch", "ai-generation"], operations: ["search-elements"], values: ["resourceSearchLevel"] },
};

function hasMeaningfulValue(value: unknown) {
  return Array.isArray(value) ? value.length > 0 : value !== undefined && value !== null && value !== "" && value !== "none" && value !== false;
}

function isGroupRelevant(title: string, values: Record<string, unknown>) {
  const trigger = groupTriggers[title];
  if (!trigger) return true;
  const main = typeof values.mainWorkType === "string" ? values.mainWorkType : "";
  const operations = Array.isArray(values.additionalOperations) ? values.additionalOperations : [];
  return trigger.main?.includes(main)
    || trigger.operations?.some((item) => operations.includes(item))
    || trigger.values?.some((key) => hasMeaningfulValue(values[key]))
    || false;
}

export function PrintDesignEditor({ service, clientName, config, pricing, suggestionsEnabled, onChange }: {
  service: QuoteService;
  clientName: string;
  config: ProfessionalServiceConfiguration;
  pricing: PricingConfiguration;
  suggestionsEnabled: boolean;
  onChange: (config: ProfessionalServiceConfiguration) => void;
}) {
  const definition = pricing.definitions.find((item) => item.serviceType === service.serviceType);
  const parameters = pricing.parameters.filter((item) => item.serviceDefinitionId === definition?.id && item.enabled);
  const visibleParameters = parameters.filter((item) => !["estimatedHours", "deliverEditableVector"].includes(item.parameterKey));
  const values = config.parameterValues;
  const effectiveValues = normalizePrintDesignEffort(values);
  const automaticEstimate = estimatePrintDesignEffort(effectiveValues);
  const editorOptions = pricing.options.filter((option) => {
    const parameter = parameters.find((item) => item.id === option.parameterId);
    return parameter?.parameterKey !== "additionalOperations" || !redundantPrintDesignOperations.has(option.value);
  });
  const effortMode = effectiveValues.estimatedHoursMode === "manual" ? "manual" : "automatic";
  const rawHours = typeof effectiveValues.estimatedHours === "number" ? effectiveValues.estimatedHours : Number(effectiveValues.estimatedHours || 0);
  const effortAmount = typeof effectiveValues.effortAmount === "number" ? effectiveValues.effortAmount : null;
  const effortUnit = ["hours", "days", "weeks"].includes(String(effectiveValues.effortUnit)) ? effectiveValues.effortUnit as EffortUnit : "hours";
  const hoursPerDay = typeof effectiveValues.hoursPerDay === "number" ? effectiveValues.hoursPerDay : 8;
  const summary = printDesignSummary(effectiveValues, parameters, pricing.options);
  const useAutomaticTime = () => {
    const next = { ...values, estimatedHours: null, effortAmount: null, estimatedHoursMode: "automatic" };
    onChange({ ...config, parameterValues: normalizePrintDesignEffort(next) });
  };

  return <div className="dynamic-editor print-design-editor">
    <section className="editor-section print-design-context">
      <div><span className="eyebrow">Diseño de estampas · definición v{definition?.version ?? 1}</span><h2>Pedido de estampa para remera</h2><p className="muted-line">DTF y sublimación. Cada opción describe este encargo y se guarda únicamente en este módulo.</p></div>
      <div className="print-design-context__facts"><span><b>Trabajo</b>{service.title}</span><span><b>Cliente</b>{clientName}</span></div>
    </section>

    <section className="complexity-guide" aria-label="Guía de complejidad">{complexityHelp.map(([title, description]) => <article key={title}><strong>{title}</strong><span>{description}</span></article>)}</section>

    <div className="print-design-groups">
      {groups.filter((group) => isGroupRelevant(group.title, effectiveValues)).map((group, index) => {
        const groupParameters = visibleParameters.filter((parameter) => group.keys.includes(parameter.parameterKey as never));
        return <details className="print-design-group" key={group.title} open={index === 0 || group.title === "Origen, urgencia y tiempo" ? true : undefined}>
          <summary><span><strong>{group.title}</strong><small>{group.description}</small></span><ChevronDown size={18} aria-hidden="true" /></summary>
          <div className="print-design-group__body">
            <DynamicFields parameters={groupParameters} options={editorOptions} values={effectiveValues} suggestionsEnabled={suggestionsEnabled} onChange={(parameterValues) => onChange({ ...config, parameterValues })} />
            {group.title === "Origen, urgencia y tiempo" && <div className="print-design-time">
              <div className="print-design-time__heading"><Clock3 size={18} /><div><strong>Tiempo del trabajo · {effortMode === "automatic" ? "estimado por la aplicación" : "definido manualmente"}</strong><span>{effortMode === "automatic" ? "Se calcula con el tipo de trabajo, la complejidad y las tareas marcadas. Podés reemplazarlo escribiendo otro tiempo." : "Tus horas reemplazan la estimación automática y se usan en los tres precios."}</span></div></div>
              <EffortInput amount={effortAmount} unit={effortUnit} hoursPerDay={hoursPerDay} estimatedHours={rawHours || null} onChange={(effort) => onChange({ ...config, parameterValues: { ...effectiveValues, effortAmount: effort.amount, effortUnit: effort.unit, hoursPerDay: effort.hoursPerDay, estimatedHours: effort.estimatedHours, estimatedHoursMode: "manual" } })} />
              <div className="print-design-time__mode"><span>{automaticEstimate ? `Estimación automática actual: ${Math.floor(automaticEstimate.hours)} h ${Math.round((automaticEstimate.hours % 1) * 60)} min` : "Elegí el tipo principal y la complejidad para estimar el tiempo."}</span>{effortMode === "manual" && automaticEstimate && <Button type="button" variant="ghost" onClick={useAutomaticTime}><RefreshCw size={15} /> Usar estimación automática</Button>}</div>
            </div>}
          </div>
        </details>;
      })}
    </div>

    <section className="editor-section"><Field label="Observaciones internas" hint="Se guardan en el módulo y no se muestran automáticamente al cliente."><textarea className="input textarea" rows={4} value={config.notes} onChange={(event) => onChange({ ...config, notes: event.target.value })} /></Field></section>

    <section className="print-design-summary" aria-live="polite"><header><FileCheck2 size={20} /><div><span className="eyebrow">Resumen automático</span><h3>Alcance marcado</h3></div></header>{summary.length > 0 ? <ul>{summary.map((item) => <li key={item}>{item}</li>)}</ul> : <div className="print-design-summary__empty"><Shirt size={22} /><span>Completá el tipo de trabajo y la complejidad para construir el resumen.</span></div>}{config.notes && <p><b>Observaciones:</b> {config.notes}</p>}</section>
  </div>;
}
