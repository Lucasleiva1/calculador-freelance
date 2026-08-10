import { convertMinor } from "./money";
import type { Currency, PriceLine, PricingContext, ProductPriceTier, ServiceResult } from "./types";

const MICRO = 1_000_000;

export interface ProductCost {
  id: string;
  name: string;
  amountMinor: number;
  currency: Currency;
  scope: "per_unit" | "batch";
  category: "material" | "production" | "packaging" | "operations" | "logistics" | "other";
}

export interface ProductConfiguration {
  quantity: number;
  costs: ProductCost[];
  wastePercent: number;
  commissionPercent: number;
  taxPercent: number;
  recommendedMarginPercent: number;
  premiumMarginPercent: number;
  selectedTier: "floor" | "recommended" | "premium";
}

export interface HybridConfiguration extends ProductConfiguration {
  serviceHours: number | null;
  serviceLabel: string;
}

export const defaultProductConfiguration = (): ProductConfiguration => ({
  quantity: 1,
  costs: [],
  wastePercent: 0,
  commissionPercent: 0,
  taxPercent: 0,
  recommendedMarginPercent: 30,
  premiumMarginPercent: 45,
  selectedTier: "recommended",
});

export const defaultHybridConfiguration = (): HybridConfiguration => ({
  ...defaultProductConfiguration(),
  serviceHours: null,
  serviceLabel: "Trabajo profesional",
});

export function validateProduct(config: ProductConfiguration): string[] {
  const issues: string[] = [];
  if (!Number.isInteger(config.quantity) || config.quantity < 1) issues.push("La cantidad debe ser un entero mayor que cero.");
  for (const cost of config.costs) {
    if (!cost.name.trim()) issues.push("Cada costo necesita un nombre.");
    if (!Number.isFinite(cost.amountMinor) || cost.amountMinor < 0) issues.push(`El costo “${cost.name || "sin nombre"}” no es válido.`);
  }
  for (const [label, value] of [
    ["merma", config.wastePercent],
    ["comisión", config.commissionPercent],
    ["impuestos", config.taxPercent],
    ["margen recomendado", config.recommendedMarginPercent],
    ["margen premium", config.premiumMarginPercent],
  ] as const) {
    if (!Number.isFinite(value) || value < 0 || value >= 100) issues.push(`El porcentaje de ${label} debe estar entre 0 y menos de 100.`);
  }
  if (config.premiumMarginPercent < config.recommendedMarginPercent) issues.push("El margen premium no puede ser menor al recomendado.");
  const sellingRate = (config.commissionPercent + config.taxPercent) / 100;
  if (sellingRate >= 1) issues.push("Comisiones e impuestos no pueden consumir el 100% del precio.");
  if (sellingRate + config.recommendedMarginPercent / 100 >= 1) issues.push("El margen recomendado no deja un precio matemáticamente sostenible.");
  if (sellingRate + config.premiumMarginPercent / 100 >= 1) issues.push("El margen premium no deja un precio matemáticamente sostenible.");
  if (config.costs.length === 0) issues.push("Agregá al menos un costo del producto.");
  return [...new Set(issues)];
}

function tier(costMinor: number, quantity: number, sellingRate: number, marginRate: number): ProductPriceTier {
  const denominatorMicros = MICRO - Math.round(sellingRate * MICRO) - Math.round(marginRate * MICRO);
  const numerator = BigInt(costMinor) * BigInt(MICRO);
  const denominator = BigInt(denominatorMicros);
  const totalMinor = Number((numerator + denominator - BigInt(1)) / denominator);
  return { totalMinor, unitMinor: Math.ceil(totalMinor / quantity), marginMicros: Math.round(marginRate * MICRO) };
}

export function calculateProduct(
  config: ProductConfiguration,
  context: PricingContext,
  additionalBatchCostMinor = 0,
  additionalLines: PriceLine[] = [],
): ServiceResult {
  const issues = validateProduct(config);
  const lines: PriceLine[] = [...additionalLines];
  let productionBeforeWaste = additionalBatchCostMinor;
  let externalBeforeWaste = 0;
  let conversionMissing = false;

  for (const cost of config.costs) {
    const converted = convertMinor(cost.amountMinor, cost.currency, context.currency, context.usdToArsMicros);
    if (converted == null) {
      conversionMissing = true;
      issues.push(`Falta el cambio USD/ARS para convertir “${cost.name}”.`);
      continue;
    }
    const total = cost.scope === "per_unit" ? converted * config.quantity : converted;
    productionBeforeWaste += total;
    externalBeforeWaste += total;
    lines.push({ label: cost.name, kind: "external", amountMinor: total, detail: cost.scope === "per_unit" ? `${config.quantity} unidades` : "Por lote" });
  }

  const wasteMinor = Math.round(externalBeforeWaste * config.wastePercent / 100);
  const productionCostMinor = productionBeforeWaste + wasteMinor;
  const externalCostsMinor = externalBeforeWaste + wasteMinor;
  if (wasteMinor > 0) lines.push({ label: "Merma prevista", kind: "percentage", amountMinor: wasteMinor, detail: `${config.wastePercent.toLocaleString("es-AR")}%` });

  if (conversionMissing || issues.length > 0 || productionCostMinor <= 0) {
    return {
      status: "incomplete", calculatedSubtotalMinor: null, suggestedSubtotalMinor: null,
      finalSubtotalMinor: null, effectiveSubtotalMinor: null, hasOverride: false,
      hours: null, externalCostsMinor, effectiveHourlyMinor: null,
      appliedMarginMicros: null, lines, issues: [...new Set(issues)], engineKind: "product",
    };
  }

  const sellingRate = (config.commissionPercent + config.taxPercent) / 100;
  const tiers = {
    floor: tier(productionCostMinor, config.quantity, sellingRate, 0),
    recommended: tier(productionCostMinor, config.quantity, sellingRate, config.recommendedMarginPercent / 100),
    premium: tier(productionCostMinor, config.quantity, sellingRate, config.premiumMarginPercent / 100),
    selected: config.selectedTier,
  };
  const selected = tiers[config.selectedTier];
  const sellingFeesMinor = Math.round(selected.totalMinor * sellingRate);
  const grossProfitMinor = selected.totalMinor - productionCostMinor - sellingFeesMinor;
  const marginMicros = selected.totalMinor > 0 ? Math.round(grossProfitMinor / selected.totalMinor * MICRO) : 0;
  const markupMicros = productionCostMinor > 0 ? Math.round(grossProfitMinor / productionCostMinor * MICRO) : 0;
  lines.push({ label: "Comisiones e impuestos", kind: "percentage", amountMinor: sellingFeesMinor, detail: `${((sellingRate) * 100).toLocaleString("es-AR")}% del precio` });
  lines.push({ label: `Margen ${config.selectedTier === "floor" ? "del piso" : config.selectedTier === "premium" ? "premium" : "recomendado"}`, kind: "margin", amountMinor: grossProfitMinor });

  return {
    status: "ready",
    calculatedSubtotalMinor: tiers.floor.totalMinor,
    suggestedSubtotalMinor: tiers.recommended.totalMinor,
    finalSubtotalMinor: selected.totalMinor,
    effectiveSubtotalMinor: selected.totalMinor,
    hasOverride: false,
    hours: null,
    externalCostsMinor,
    effectiveHourlyMinor: null,
    appliedMarginMicros: selected.marginMicros,
    lines,
    issues: [],
    engineKind: "product",
    pricingTiers: tiers,
    productMetrics: {
      quantity: config.quantity,
      costUnitMinor: Math.ceil(productionCostMinor / config.quantity),
      productionCostMinor,
      revenueMinor: selected.totalMinor,
      grossProfitMinor,
      marginMicros,
      markupMicros,
      sellingFeesMinor,
    },
  };
}

export function calculateHybrid(config: HybridConfiguration, context: PricingContext): ServiceResult {
  const hours = config.serviceHours;
  const issues: string[] = [];
  if (hours == null || !Number.isFinite(hours) || hours <= 0) issues.push("Indicá horas válidas para la parte profesional.");
  if (context.hourlyRateMinor == null) issues.push(`Configurá tu tarifa profesional en ${context.currency}.`);
  if (issues.length > 0 || hours == null || context.hourlyRateMinor == null) {
    const product = calculateProduct(config, context);
    return { ...product, status: "incomplete", finalSubtotalMinor: null, effectiveSubtotalMinor: null, hours: hours && hours > 0 ? hours : null, issues: [...new Set([...product.issues, ...issues])], engineKind: "hybrid" };
  }
  const serviceCost = Math.round(hours * context.hourlyRateMinor);
  const product = calculateProduct(config, context, serviceCost, [{ label: config.serviceLabel || "Trabajo profesional", kind: "base", amountMinor: serviceCost, detail: `${hours.toLocaleString("es-AR")} h` }]);
  return { ...product, hours, effectiveHourlyMinor: product.finalSubtotalMinor == null ? null : Math.round((product.finalSubtotalMinor - product.externalCostsMinor) / hours), engineKind: "hybrid" };
}
