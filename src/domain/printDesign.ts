import type { ParameterOption, ServiceParameter } from "./types";

export type PrintDesignEffortMode = "automatic" | "manual";
export type PrintDesignComplexity = "basic" | "intermediate" | "complex";
export type PrintDesignPriceKind = "sustainable" | "market" | "international";

export interface PrintDesignPriceSelection {
  kind: PrintDesignPriceKind;
  amountMinor: number;
  currency: "ARS" | "USD";
  selectedAt: string;
  marketSnapshotId: string | null;
}

export interface PrintDesignEffortEstimate {
  hours: number;
  complexity: PrintDesignComplexity;
  score: number;
  factors: string[];
}

export const printDesignTaskOptions = [
  ["remove-background", "Quitar fondo"],
  ["improve-quality", "Mejorar calidad / resolución"],
  ["reconstruct-image", "Reconstruir o completar imagen"],
  ["vectorize-simple", "Vectorizar texto o gráfico simple"],
  ["optimize-image", "Ajustar y optimizar imagen"],
  ["adapt-composition", "Adaptar composición para estampa"],
  ["grunge-borders", "Crear bordes / grunge / integrar imagen"],
  ["halftone", "Aplicar semitono"],
  ["ai-elements", "Generar o reconstruir elementos con IA"],
  ["design-from-scratch", "Crear diseño desde cero"],
] as const;

export const deliveryExtraOptions = [
  ["psd", "PSD editable"],
  ["ai-vector", "AI / vector editable"],
  ["extra-versions", "Versiones adicionales"],
  ["extra-sizes", "Tamaños o adaptaciones adicionales"],
] as const;

const taskScores: Record<string, number> = {
  "remove-background": 1,
  "vectorize-simple": 1,
  "optimize-image": 1,
  "improve-quality": 2,
  "adapt-composition": 2,
  "grunge-borders": 2,
  halftone: 2,
  "ai-elements": 2,
  "reconstruct-image": 3,
  "design-from-scratch": 4,
};

const taskHours: Record<string, number> = {
  "remove-background": 0.75,
  "improve-quality": 1,
  "reconstruct-image": 2.5,
  "vectorize-simple": 1.5,
  "optimize-image": 0.75,
  "adapt-composition": 1.5,
  "grunge-borders": 1,
  halftone: 0.75,
  "ai-elements": 1.5,
  "design-from-scratch": 4,
};

export const printDesignDeliveryHours: Record<string, number> = {
  psd: 0.5,
  "ai-vector": 0.5,
  "extra-versions": 0.75,
  "extra-sizes": 0.75,
};

function stringValue(value: unknown) {
  return typeof value === "string" ? value : "";
}

function numericValue(value: unknown) {
  const parsed = typeof value === "number" ? value : typeof value === "string" && value.trim() ? Number(value.replace(",", ".")) : NaN;
  return Number.isFinite(parsed) ? parsed : 0;
}

function uniqueList(value: unknown) {
  return Array.isArray(value) ? [...new Set(value.filter((item): item is string => typeof item === "string"))] : [];
}

function mappedLegacyTasks(values: Record<string, unknown>) {
  const legacy = new Set(uniqueList(values.additionalOperations));
  const tasks = new Set(uniqueList(values.workTasks));
  const main = stringValue(values.mainWorkType);
  const mappings: Array<[string[], string]> = [
    [["remove-background", "clean-edges", "background-simple", "background-complex"], "remove-background"],
    [["fix-resolution", "improve-ai", "image-cleaning"], "improve-quality"],
    [["reconstruct-missing", "redraw-partial", "redraw-full", "restore-image"], "reconstruct-image"],
    [["vectorize", "vector-basic", "vector-corrected"], "vectorize-simple"],
    [["adjust-colors", "change-colors", "black-white"], "optimize-image"],
    [["compose", "composition-simple", "composition-medium", "composition-complex", "adapt-existing"], "adapt-composition"],
    [["generate-ai", "inpainting", "ai-generation", "ai-image-edit"], "ai-elements"],
    [["apply-halftone"], "halftone"],
    [["design-from-scratch"], "design-from-scratch"],
  ];
  for (const [oldValues, next] of mappings) {
    if (oldValues.includes(main) || oldValues.some((value) => legacy.has(value))) tasks.add(next);
  }
  if (stringValue(values.halftoneLevel) && stringValue(values.halftoneLevel) !== "none") tasks.add("halftone");
  return [...tasks].filter((task) => task in taskHours);
}

/** Migra borradores anteriores y elimina combinaciones condicionales imposibles. */
export function canonicalizePrintDesignValues(values: Record<string, unknown>): Record<string, unknown> {
  const next = { ...values };
  const tasks = mappedLegacyTasks(values);
  const oldOrigin = stringValue(values.designOrigin);
  const oldMain = stringValue(values.mainWorkType);

  if (typeof next.hasReference !== "boolean" && oldOrigin) {
    const contradictory = oldOrigin === "from-scratch" && oldMain && !["design-from-scratch", "ai-generation"].includes(oldMain);
    if (contradictory) {
      delete next.hasReference;
    } else next.hasReference = oldOrigin !== "from-scratch";
  }
  if (next.hasReference === false && !tasks.includes("design-from-scratch")) tasks.push("design-from-scratch");
  if (next.hasReference === false) delete next.materialType;

  if (!stringValue(next.clientTier)) {
    const tier = { individual: "small", venture: "small", "small-brand": "small", "medium-brand": "medium" }[stringValue(values.clientType)];
    if (tier) next.clientTier = tier;
  }
  if (!stringValue(next.materialType)) {
    next.materialType = ({ excellent: "ready", good: "ready", regular: "low-quality", bad: "low-quality", "very-bad": "low-quality" } as Record<string, string>)[stringValue(values.inputQuality)];
  }
  if (next.hasReference === false) delete next.materialType;
  if (!stringValue(next.productType) && (oldMain || oldOrigin)) next.productType = "shirt";
  if (!stringValue(next.printSystem)) {
    const output = uniqueList(values.printOutput);
    const legacyOperations = uniqueList(values.additionalOperations);
    next.printSystem = output.includes("dtf") || legacyOperations.includes("prepare-dtf") ? "dtf" : output.includes("sublimation") || legacyOperations.includes("prepare-sublimation") ? "sublimation" : "design-only";
  }
  if (next.printSystem !== "sublimation") delete next.sublimationFitsA4;
  if (next.productType !== "other") delete next.otherProduct;

  if (!stringValue(next.complexity)) {
    next.complexity = ({ basic: "basic", medium: "intermediate", high: "complex", premium: "complex" } as Record<string, string>)[stringValue(values.complexity)];
  } else if (next.complexity === "medium") next.complexity = "intermediate";
  else if (["high", "premium"].includes(String(next.complexity))) next.complexity = "complex";

  if (!Array.isArray(next.deliveryExtras)) {
    const delivery = new Set<string>();
    const editable = stringValue(values.editableDelivery);
    if (editable === "psd") delivery.add("psd");
    if (["ai", "svg", "other"].includes(editable)) delivery.add("ai-vector");
    if (stringValue(values.variantLevel) && stringValue(values.variantLevel) !== "none") delivery.add("extra-versions");
    if (numericValue(values.finalSizeCount) > 1) delivery.add("extra-sizes");
    next.deliveryExtras = [...delivery];
  }
  next.workTasks = tasks;
  const stableKeys = [
    "hasReference", "materialType", "clientTier", "productType", "otherProduct", "garmentTone",
    "printSystem", "sublimationFitsA4", "workTasks", "complexity", "complexityMode",
    "estimatedHours", "estimatedHoursMode", "effortAmount", "effortUnit", "hoursPerDay",
    "deliveryExtras", "priceSelection",
  ];
  return Object.fromEntries(stableKeys.filter((key) => next[key] !== undefined).map((key) => [key, next[key]]));
}

export function suggestedPrintDesignComplexity(values: Record<string, unknown>) {
  const canonical = canonicalizePrintDesignValues(values);
  const tasks = uniqueList(canonical.workTasks);
  let score = tasks.reduce((sum, task) => sum + (taskScores[task] ?? 0), 0);
  if (canonical.hasReference === false && !tasks.includes("design-from-scratch")) score += 4;
  if (["low-quality", "screenshot"].includes(stringValue(canonical.materialType))) score += 1;
  if (canonical.materialType === "reference-only") score += 2;
  const forcedComplex = canonical.hasReference === false || tasks.includes("design-from-scratch");
  const complexity: PrintDesignComplexity = forcedComplex || score >= 7 ? "complex" : score >= 3 ? "intermediate" : "basic";
  return { complexity, score };
}

export function estimatePrintDesignEffort(values: Record<string, unknown>): PrintDesignEffortEstimate {
  const canonical = canonicalizePrintDesignValues(values);
  const tasks = uniqueList(canonical.workTasks);
  const suggested = suggestedPrintDesignComplexity(canonical);
  const complexity = canonical.complexityMode === "manual" && ["basic", "intermediate", "complex"].includes(stringValue(canonical.complexity))
    ? canonical.complexity as PrintDesignComplexity
    : suggested.complexity;
  let hours = 0.5;
  const factors = ["Base operativa: 0,5 h"];
  for (const task of tasks) {
    const added = taskHours[task] ?? 0;
    if (added > 0) {
      hours += added;
      factors.push(`${task}: +${added.toLocaleString("es-AR")} h`);
    }
  }
  if (canonical.hasReference === false) {
    hours += 2;
    factors.push("Conceptualización sin referencia: +2 h");
  }
  const multiplier = { basic: 0.9, intermediate: 1, complex: 1.35 }[complexity];
  hours = Math.max(0.5, Math.round(hours * multiplier * 4) / 4);
  factors.push(`Complejidad ${complexity}: × ${multiplier.toLocaleString("es-AR")}`);
  return { hours, complexity, score: suggested.score, factors };
}

export function normalizePrintDesignEffort(values: Record<string, unknown>): Record<string, unknown> {
  const canonical = canonicalizePrintDesignValues(values);
  const suggested = suggestedPrintDesignComplexity(canonical);
  if (canonical.complexityMode !== "manual") {
    canonical.complexity = suggested.complexity;
    canonical.complexityMode = "automatic";
  }
  const currentHours = numericValue(canonical.estimatedHours);
  if (canonical.estimatedHoursMode === "manual" || (canonical.estimatedHoursMode == null && currentHours > 0)) {
    return { ...canonical, estimatedHours: currentHours, estimatedHoursMode: "manual" satisfies PrintDesignEffortMode };
  }
  const estimate = estimatePrintDesignEffort(canonical);
  return { ...canonical, estimatedHours: estimate.hours, estimatedHoursMode: "automatic" satisfies PrintDesignEffortMode };
}

export function printDesignPreparationRate(values: Record<string, unknown>) {
  const system = stringValue(values.printSystem);
  return system === "dtf" || (system === "sublimation" && values.sublimationFitsA4 === false) ? 0.15 : 0;
}

export function printDesignExtraHours(values: Record<string, unknown>) {
  return uniqueList(values.deliveryExtras).reduce((sum, extra) => sum + (printDesignDeliveryHours[extra] ?? 0), 0);
}

export function printDesignWorkClass(values: Record<string, unknown>) {
  const canonical = canonicalizePrintDesignValues(values);
  const tasks = uniqueList(canonical.workTasks);
  if (canonical.hasReference === false || tasks.includes("design-from-scratch")) return "original";
  if (tasks.some((task) => ["adapt-composition", "grunge-borders", "ai-elements", "reconstruct-image"].includes(task))) return "adaptation";
  return "preparation";
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
  const canonical = normalizePrintDesignEffort(values);
  const byKey = (key: string) => parameters.find((parameter) => parameter.parameterKey === key);
  const label = (key: string) => optionLabel(byKey(key), canonical[key], options);
  const hours = numericValue(canonical.estimatedHours);
  const prep = printDesignPreparationRate(canonical);
  return [
    typeof canonical.hasReference === "boolean" && `Referencia: ${canonical.hasReference ? "Sí" : "No · concepto desde cero"}`,
    label("materialType") && canonical.hasReference !== false && `Material: ${label("materialType")}`,
    label("clientTier") && `Cliente: ${label("clientTier")}`,
    label("productType") && `Producto: ${label("productType")}${canonical.productType === "other" && canonical.otherProduct ? ` · ${canonical.otherProduct}` : ""}`,
    label("garmentTone") && `Prenda: ${label("garmentTone")}`,
    label("printSystem") && `Sistema: ${label("printSystem")}`,
    canonical.printSystem === "sublimation" && `Formato: ${canonical.sublimationFitsA4 === true ? "1 hoja A4" : canonical.sublimationFitsA4 === false ? "Varias hojas A4" : "Pendiente"}`,
    prep > 0 ? `Preparación técnica: +${prep * 100}%` : label("printSystem") ? "Preparación técnica: sin recargo" : "",
    label("workTasks") && `Trabajo: ${label("workTasks")}`,
    `Complejidad: ${{ basic: "Básica", intermediate: "Intermedia", complex: "Compleja" }[String(canonical.complexity)] ?? "Pendiente"}`,
    hours > 0 && `Tiempo: ${hours.toLocaleString("es-AR")} h`,
    `Entrega: archivo final${label("deliveryExtras") ? ` + ${label("deliveryExtras")}` : ""}`,
  ].filter(Boolean) as string[];
}
