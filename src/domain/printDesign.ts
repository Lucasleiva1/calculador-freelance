import type { ParameterOption, ServiceParameter } from "./types";

export type PrintDesignEffortMode = "automatic" | "manual";

export interface PrintDesignEffortEstimate {
  hours: number;
  factors: string[];
}

export const redundantPrintDesignOperations = new Set([
  "prepare-dtf",
  "prepare-sublimation",
  "apply-halftone",
  "export-versions",
  "deliver-editable",
]);

const mainWorkHours: Record<string, number> = {
  "background-simple": 0.75,
  "background-complex": 2,
  "image-cleaning": 1.5,
  "vector-basic": 2,
  "vector-corrected": 3.5,
  "redraw-partial": 4,
  "redraw-full": 7,
  "composition-simple": 2.5,
  "composition-medium": 4.5,
  "composition-complex": 7,
  "design-from-scratch": 6.5,
  "adapt-existing": 3,
  "restore-image": 4.5,
  "ai-generation": 3,
  "ai-image-edit": 3,
  "print-preparation": 1.25,
};

const severityHours: Record<string, Record<string, number>> = {
  inputQuality: { excellent: 0, good: 0, regular: 0.5, bad: 1.5, "very-bad": 2.5 },
  backgroundLevel: { none: 0, simple: 0.25, medium: 0.75, complex: 1.5, "very-complex": 2.5 },
  restorationLevel: { none: 0, light: 0.5, medium: 1, high: 2, "very-high": 3 },
  vectorizationLevel: { none: 0, "basic-auto": 0.5, "manual-correction": 1.5, advanced: 2.5, "redraw-partial": 3, "redraw-full": 5 },
  compositionLevel: { none: 0, simple: 0.5, medium: 1.5, complex: 3 },
  aiLevel: { none: 0, simple: 0.25, medium: 0.75, advanced: 1.5 },
  typographyLevel: { none: 0, simple: 0.25, adjusted: 0.5, "font-search": 1, lettering: 2, advanced: 2.5 },
  colorLevel: { none: 0, light: 0.25, medium: 0.75, advanced: 1.5 },
  halftoneLevel: { none: 0, simple: 0.5, medium: 1, advanced: 2 },
  resourceSearchLevel: { none: 0, simple: 0.5, medium: 1, wide: 2 },
};

const countedHours: Record<string, Record<string, number>> = {
  initialProposals: { one: 0, two: 0.75, three: 1.5, "four-plus": 2.5 },
  includedRevisions: { zero: 0, one: 0.5, two: 1, three: 1.5, "four-plus": 2.5 },
  variantLevel: { none: 0, one: 0.5, two: 1, three: 1.5, "four-plus": 2.5 },
};

const actionListHours: Record<string, number> = {
  inputConditions: 0.15,
  backgroundDetails: 0.2,
  restorationTasks: 0.2,
  vectorizationFeatures: 0.2,
  compositionFeatures: 0.2,
  aiActions: 0.25,
  typographyActions: 0.15,
  colorActions: 0.15,
  printActions: 0.15,
  halftoneDetails: 0.2,
  resourceTypes: 0.15,
  variantTypes: 0.2,
};

function stringValue(value: unknown) {
  return typeof value === "string" ? value : "";
}

function listLength(value: unknown) {
  return Array.isArray(value) ? value.length : 0;
}

function numericValue(value: unknown) {
  const parsed = typeof value === "number" ? value : typeof value === "string" && value.trim() ? Number(value.replace(",", ".")) : NaN;
  return Number.isFinite(parsed) ? parsed : 0;
}

function uniqueList(value: unknown) {
  return Array.isArray(value) ? [...new Set(value.filter((item): item is string => typeof item === "string"))] : [];
}

/** Une las elecciones que el formulario anterior repetía en dos lugares. */
export function canonicalizePrintDesignValues(values: Record<string, unknown>) {
  const next = { ...values };
  const mainWork = stringValue(values.mainWorkType);
  const operations = uniqueList(values.additionalOperations);
  const output = new Set(uniqueList(values.printOutput));
  if (operations.includes("prepare-dtf")) output.add("dtf");
  if (operations.includes("prepare-sublimation")) output.add("sublimation");
  if (output.size > 0) next.printOutput = [...output];
  if (operations.includes("apply-halftone") && !stringValue(values.halftoneLevel)) next.halftoneLevel = "simple";
  if (operations.includes("export-versions") && !stringValue(values.variantLevel)) next.variantLevel = "one";
  if ((operations.includes("deliver-editable") || values.deliverEditableVector === true) && !stringValue(values.editableDelivery)) next.editableDelivery = "other";
  if (!stringValue(values.urgency)) next.urgency = "normal";
  if (!stringValue(values.designOrigin)) {
    next.designOrigin = ["design-from-scratch", "ai-generation"].includes(mainWork)
      ? "from-scratch"
      : ["redraw-partial", "redraw-full", "restore-image"].includes(mainWork)
        ? "reference-to-redo"
        : ["vector-corrected", "adapt-existing", "ai-image-edit"].includes(mainWork)
          ? "base-to-correct"
          : "ready";
  }
  next.additionalOperations = operations.filter((value) => !redundantPrintDesignOperations.has(value));
  delete next.deliverEditableVector;
  return next;
}

/**
 * Estima esfuerzo operativo, no dinero. La tarifa sigue viniendo exclusivamente
 * de la economía de Diseño de estampas y las fuentes conservan sus propios precios.
 */
export function estimatePrintDesignEffort(values: Record<string, unknown>): PrintDesignEffortEstimate | null {
  const mainWork = stringValue(values.mainWorkType);
  const complexity = stringValue(values.complexity);
  const base = mainWorkHours[mainWork];
  if (base == null || !complexity) return null;

  const multiplier = { basic: 0.85, medium: 1, high: 1.5, premium: 2 }[complexity] ?? 1;
  let hours = base * multiplier;
  const factors = [`Trabajo principal: ${base.toLocaleString("es-AR")} h`, `Complejidad ${complexity}: × ${multiplier.toLocaleString("es-AR")}`];

  const operations = Math.min(5, listLength(values.additionalOperations) * 0.35);
  if (operations > 0) { hours += operations; factors.push(`Tareas adicionales: +${operations.toLocaleString("es-AR")} h`); }

  for (const [key, scale] of Object.entries(severityHours)) {
    const added = scale[stringValue(values[key])] ?? 0;
    if (added > 0) { hours += added; factors.push(`${key}: +${added.toLocaleString("es-AR")} h`); }
  }
  for (const [key, scale] of Object.entries(countedHours)) {
    const added = scale[stringValue(values[key])] ?? 0;
    if (added > 0) { hours += added; factors.push(`${key}: +${added.toLocaleString("es-AR")} h`); }
  }

  for (const [key, perSelection] of Object.entries(actionListHours)) {
    const added = Math.min(2.5, listLength(values[key]) * perSelection);
    if (added > 0) { hours += added; factors.push(`${key}: +${added.toLocaleString("es-AR")} h`); }
  }

  const elementBandHours = { one: 0, "two-three": 0.5, "four-six": 1, "seven-ten": 1.75, "more-ten": 2.5 }[stringValue(values.elementCountBand)] ?? 0;
  if (elementBandHours > 0) { hours += elementBandHours; factors.push(`Cantidad de elementos: +${elementBandHours.toLocaleString("es-AR")} h`); }
  const editableHours = { none: 0, svg: 0.25, psd: 0.5, ai: 0.5, other: 0.5 }[stringValue(values.editableDelivery)] ?? 0;
  if (editableHours > 0) { hours += editableHours; factors.push(`Preparación de editable: +${editableHours.toLocaleString("es-AR")} h`); }
  const generationHours = Math.min(2.5, Math.max(0, numericValue(values.aiGenerationCount) - 1) * 0.25);
  if (generationHours > 0) { hours += generationHours; factors.push(`Pruebas con IA: +${generationHours.toLocaleString("es-AR")} h`); }
  const extraRevisionHours = values.hasExtraRevisions === true ? Math.min(4, numericValue(values.extraRevisionValue) * 0.5) : 0;
  if (extraRevisionHours > 0) { hours += extraRevisionHours; factors.push(`Revisiones extra: +${extraRevisionHours.toLocaleString("es-AR")} h`); }

  const elementCount = Math.max(0, numericValue(values.vectorizationElementCount) + numericValue(values.compositionElementCount) + numericValue(values.resourceCount) - 3);
  if (elementCount > 0) {
    const added = Math.min(3, elementCount * 0.2);
    hours += added;
    factors.push(`Elementos adicionales: +${added.toLocaleString("es-AR")} h`);
  }
  const extraOutputs = Math.max(0, listLength(values.printLocations) + numericValue(values.finalSizeCount) - 2);
  if (extraOutputs > 0) {
    const added = Math.min(2.5, extraOutputs * 0.35);
    hours += added;
    factors.push(`Salidas adicionales: +${added.toLocaleString("es-AR")} h`);
  }

  return { hours: Math.min(160, Math.max(0.5, Math.round(hours * 4) / 4)), factors };
}

export function normalizePrintDesignEffort(values: Record<string, unknown>) {
  const canonical = canonicalizePrintDesignValues(values);
  const currentHours = numericValue(canonical.estimatedHours);
  const mode = canonical.estimatedHoursMode;
  if (mode === "manual" || (mode == null && currentHours > 0)) {
    return { ...canonical, estimatedHours: currentHours, estimatedHoursMode: "manual" satisfies PrintDesignEffortMode };
  }
  const estimate = estimatePrintDesignEffort(canonical);
  if (!estimate) return canonical;
  return { ...canonical, estimatedHours: estimate.hours, estimatedHoursMode: "automatic" satisfies PrintDesignEffortMode };
}

function optionLabel(parameter: ServiceParameter | undefined, value: unknown, options: ParameterOption[]) {
  if (!parameter) return "";
  const labels = options.filter((option) => option.parameterId === parameter.id);
  if (Array.isArray(value)) return value.map((item) => labels.find((option) => option.value === item)?.label ?? String(item)).join(", ");
  if (typeof value === "boolean") return value ? "Sí" : "No";
  if (value == null || value === "") return "";
  return labels.find((option) => option.value === value)?.label ?? String(value);
}

export function printDesignSummary(values: Record<string, unknown>, parameters: ServiceParameter[], options: ParameterOption[]) {
  const byKey = (key: string) => parameters.find((parameter) => parameter.parameterKey === key);
  const label = (key: string) => optionLabel(byKey(key), values[key], options);
  const hours = typeof values.estimatedHours === "number" ? values.estimatedHours : Number(values.estimatedHours || 0);
  const time = hours > 0 ? `${Math.floor(hours)} h ${Math.round((hours % 1) * 60)} min` : "";
  return [
    label("mainWorkType") && `Trabajo: ${label("mainWorkType")}`,
    label("complexity") && `Complejidad: ${label("complexity")}`,
    label("additionalOperations") && `Tareas: ${label("additionalOperations")}`,
    label("printOutput") && `Salida: ${label("printOutput")}`,
    label("halftoneLevel") && label("halftoneLevel") !== "No requiere" && `Semitono: ${label("halftoneLevel")}`,
    label("editableDelivery") && `Editable: ${label("editableDelivery")}`,
    label("includedRevisions") && `Revisiones: ${label("includedRevisions")}`,
    label("variantLevel") && `Variantes: ${label("variantLevel")}`,
    time && `Tiempo: ${time}`,
  ].filter(Boolean) as string[];
}
