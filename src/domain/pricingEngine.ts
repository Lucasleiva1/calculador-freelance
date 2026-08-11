import { convertMinor } from "./money";
import type {
  AppSettings, Currency, EconomicProfile, ParameterOption, PriceLine, PricingConfiguration,
  PricingRule, PricingSnapshot, QuoteService, ServiceDefinition, ServiceParameter,
  ServiceResult, ServiceType,
} from "./types";
import type { ExternalCost } from "./video";

const MICRO = 1_000_000;

export function applySuggestedDefaults(values: Record<string, unknown>, parameters: ServiceParameter[], suggestionsEnabled: boolean) {
  if (!suggestionsEnabled) return values;
  const next = { ...values };
  for (const parameter of parameters) {
    if (!parameter.suggestionEnabled || next[parameter.parameterKey] !== undefined || !parameter.defaultValueJson) continue;
    try { next[parameter.parameterKey] = JSON.parse(parameter.defaultValueJson); } catch { /* La persistencia valida JSON antes de aceptarlo. */ }
  }
  return next;
}

export interface EngineInput {
  serviceType: ServiceType;
  currency: Currency;
  parameterValues: Record<string, unknown>;
  externalCosts?: ExternalCost[];
  fixedUrgencyMinor?: number;
  finalOverrideMinor?: number | null;
  hasOverride?: boolean;
  settings: AppSettings;
  pricing: PricingConfiguration;
}

export interface SustainableRateResult {
  rateMinor: number | null;
  monthlyRequiredMinor: number | null;
  issues: string[];
}

export function calculateSustainableRate(profile: EconomicProfile | null): SustainableRateResult {
  if (!profile) return { rateMinor: null, monthlyRequiredMinor: null, issues: ["Falta el perfil económico."] };
  const issues: string[] = [];
  const income = profile.monthlyIncomeTargetMinor;
  const expenses = profile.monthlyExpensesMinor;
  const hours = profile.billableHoursMicros == null ? null : profile.billableHoursMicros / MICRO;
  const reserve = (profile.reserveTaxMicros ?? 0) / MICRO;
  if (income == null) issues.push("Indicá tu objetivo mensual.");
  if (expenses == null) issues.push("Indicá tus gastos mensuales.");
  if (hours == null || hours <= 0) issues.push("Indicá horas facturables mensuales válidas.");
  if (reserve < 0 || reserve >= 1) issues.push("La reserva debe ser menor al 100%.");
  if (issues.length > 0 || income == null || expenses == null || hours == null) {
    return { rateMinor: null, monthlyRequiredMinor: null, issues };
  }
  const monthlyRequiredMinor = Math.round((income + expenses) / (1 - reserve));
  return { rateMinor: Math.round(monthlyRequiredMinor / hours), monthlyRequiredMinor, issues: [] };
}

export function activeHourlyRate(profile: EconomicProfile | null): number | null {
  return profile?.manualHourlyRateMinor ?? calculateSustainableRate(profile).rateMinor;
}

function numeric(value: unknown): number | null {
  if (typeof value === "number" && Number.isFinite(value)) return value;
  if (typeof value === "string" && value.trim() && Number.isFinite(Number(value.replace(",", ".")))) return Number(value.replace(",", "."));
  return null;
}

function matchesRule(rule: PricingRule, parameters: ServiceParameter[], options: ParameterOption[], values: Record<string, unknown>) {
  if (!rule.parameterId) return true;
  const parameter = parameters.find((item) => item.id === rule.parameterId);
  if (!parameter) return false;
  const value = values[parameter.parameterKey];
  if (rule.optionId) {
    const option = options.find((item) => item.id === rule.optionId);
    if (!option) return false;
    return Array.isArray(value) ? value.includes(option.value) : value === option.value;
  }
  if (Array.isArray(value)) return value.length > 0;
  return value !== null && value !== undefined && value !== "" && value !== false;
}

function ruleAmount(rule: PricingRule, currency: Currency) {
  return currency === "ARS" ? rule.amountArsMinor : rule.amountUsdMinor;
}

function withMargin(costMinor: number, marginMicros: number | null) {
  if (!marginMicros) return costMinor;
  return Math.round(costMinor / (1 - marginMicros / MICRO));
}

function marginForStrategy(definition: ServiceDefinition, strategy: AppSettings["suggestionStrategy"]) {
  if (strategy === "competitive") return definition.competitiveMarginMicros;
  if (strategy === "premium") return definition.premiumMarginMicros;
  return definition.balancedMarginMicros;
}

export function runPricingEngine(input: EngineInput): ServiceResult {
  const definition = input.pricing.definitions.find((item) => item.serviceType === input.serviceType);
  if (!definition || !definition.enabled) return emptyResult("El servicio no tiene una definición activa.");
  const parameters = input.pricing.parameters.filter((item) => item.serviceDefinitionId === definition.id && item.enabled);
  const options = input.pricing.options.filter((item) => parameters.some((parameter) => parameter.id === item.parameterId) && item.enabled);
  const rules = input.pricing.rules.filter((item) => item.serviceDefinitionId === definition.id && item.enabled).sort((a, b) => a.sortOrder - b.sortOrder);
  const profile = input.pricing.economicProfiles.find((item) => item.currency === input.currency) ?? null;
  const hourlyRateMinor = activeHourlyRate(profile);
  const issues: string[] = [];
  const lines: PriceLine[] = [];
  let hours = numeric(input.parameterValues.estimatedHours) ?? 0;
  let costMinor = 0;
  let externalCostsMinor = 0;
  for (const parameter of parameters.filter((item) => item.required)) {
    const value = input.parameterValues[parameter.parameterKey];
    if (value == null || value === "" || (Array.isArray(value) && value.length === 0)) issues.push(`Completá “${parameter.label}”.`);
  }
  if (hours > 0 && hourlyRateMinor != null) {
    const base = Math.round(hours * hourlyRateMinor);
    costMinor += base;
    lines.push({ label: "Horas base × tarifa sostenible", kind: "base", amountMinor: base, detail: `${hours.toLocaleString("es-AR")} h` });
  } else if (hourlyRateMinor == null) issues.push(`Configurá tu economía en ${input.currency}.`);

  for (const rule of rules) {
    if (!matchesRule(rule, parameters, options, input.parameterValues)) continue;
    const amount = ruleAmount(rule, input.currency);
    if (rule.ruleType === "hours") {
      const extra = (rule.numericValueMicros ?? 0) / MICRO;
      if (extra > 0 && hourlyRateMinor != null) {
        hours += extra;
        const lineAmount = Math.round(extra * hourlyRateMinor);
        costMinor += lineAmount;
        lines.push({ id: rule.id, label: rule.name, kind: "hours", amountMinor: lineAmount, detail: `+${extra.toLocaleString("es-AR")} h` });
      }
    } else if (rule.ruleType === "fixed_amount" || rule.ruleType === "external_cost") {
      if (amount != null) {
        costMinor += amount;
        if (rule.ruleType === "external_cost") externalCostsMinor += amount;
        lines.push({ id: rule.id, label: rule.name, kind: rule.ruleType, amountMinor: amount });
      } else issues.push(`La regla “${rule.name}” no tiene importe en ${input.currency}.`);
    } else if (rule.ruleType === "per_unit") {
      const quantityParameter = parameters.find((item) => item.id === rule.quantityParameterId);
      const quantity = numeric(quantityParameter ? input.parameterValues[quantityParameter.parameterKey] : null) ?? 0;
      const coefficient = rule.numericValueMicros == null ? 1 : rule.numericValueMicros / MICRO;
      if (amount != null && quantity >= 0) {
        const lineAmount = Math.round(amount * quantity * coefficient);
        costMinor += lineAmount;
        lines.push({ id: rule.id, label: rule.name, kind: "per_unit", amountMinor: lineAmount, detail: `${quantity} u.` });
      } else issues.push(`La regla “${rule.name}” necesita importe y cantidad.`);
    } else if (rule.ruleType === "percentage") {
      const percentage = (rule.numericValueMicros ?? 0) / MICRO;
      const lineAmount = Math.round(costMinor * percentage);
      costMinor += lineAmount;
      lines.push({ id: rule.id, label: rule.name, kind: "percentage", amountMinor: lineAmount, detail: `${(percentage * 100).toLocaleString("es-AR")}% de recargo` });
    } else if (rule.ruleType === "multiplier") {
      const multiplier = (rule.numericValueMicros ?? MICRO) / MICRO;
      const before = costMinor;
      costMinor = Math.round(costMinor * multiplier);
      lines.push({ id: rule.id, label: rule.name, kind: "multiplier", amountMinor: costMinor - before, detail: `× ${multiplier.toLocaleString("es-AR")}` });
    }
  }

  if (hours <= 0) issues.push("Indicá horas estimadas o agregá una regla de horas aplicable.");

  for (const cost of input.externalCosts ?? []) {
    const converted = convertMinor(cost.amountMinor, cost.currency, input.currency, input.settings.usdToArsMicros);
    if (converted == null) issues.push(`Falta el cambio USD/ARS para convertir “${cost.name}”.`);
    else {
      externalCostsMinor += converted;
      costMinor += converted;
      lines.push({ label: cost.name || "Costo externo", kind: "external", amountMinor: converted, detail: cost.currency === input.currency ? undefined : `Convertido desde ${cost.currency}` });
    }
  }
  if ((input.fixedUrgencyMinor ?? 0) > 0) {
    costMinor += input.fixedUrgencyMinor ?? 0;
    lines.push({ label: "Urgencia fija", kind: "fixed_amount", amountMinor: input.fixedUrgencyMinor ?? 0 });
  }

  const complete = issues.length === 0 && hourlyRateMinor != null && hours > 0;
  const economicMargin = profile?.desiredMarginMicros ?? null;
  const calculated = complete ? withMargin(costMinor, economicMargin) : null;
  if (calculated != null && economicMargin) lines.push({ label: "Margen económico deseado", kind: "margin", amountMinor: calculated - costMinor, detail: `${(economicMargin / 10_000).toLocaleString("es-AR")}%` });

  const canSuggest = complete && input.settings.suggestionsEnabled && definition.suggestionsEnabled;
  const strategyMargin = marginForStrategy(definition, input.settings.suggestionStrategy) ?? economicMargin;
  const floor = calculated;
  const recommended = complete && floor != null
    ? Math.max(floor, withMargin(costMinor, definition.balancedMarginMicros ?? economicMargin))
    : null;
  const premium = complete && recommended != null
    ? Math.max(recommended, withMargin(costMinor, definition.premiumMarginMicros ?? definition.balancedMarginMicros ?? economicMargin))
    : null;
  const selectedTier: "floor" | "recommended" | "premium" = input.settings.suggestionStrategy === "premium" ? "premium" : input.settings.suggestionStrategy === "competitive" ? "floor" : "recommended";
  const pricingTiers = floor != null && recommended != null && premium != null ? {
    floor: { unitMinor: floor, totalMinor: floor, marginMicros: economicMargin ?? 0 },
    recommended: { unitMinor: recommended, totalMinor: recommended, marginMicros: definition.balancedMarginMicros ?? economicMargin ?? 0 },
    premium: { unitMinor: premium, totalMinor: premium, marginMicros: definition.premiumMarginMicros ?? definition.balancedMarginMicros ?? economicMargin ?? 0 },
    selected: selectedTier,
  } : undefined;
  const suggested = canSuggest && pricingTiers ? pricingTiers[selectedTier].totalMinor : null;
  const hasOverride = Boolean(input.hasOverride && input.finalOverrideMinor != null);
  const final = hasOverride ? input.finalOverrideMinor ?? null : suggested ?? calculated;
  if (hasOverride && final != null) lines.push({ label: "Precio final manual", kind: "override", amountMinor: final - (suggested ?? calculated ?? 0) });
  return {
    status: complete ? "ready" : "incomplete",
    calculatedSubtotalMinor: calculated,
    suggestedSubtotalMinor: suggested,
    finalSubtotalMinor: final,
    effectiveSubtotalMinor: final,
    hasOverride,
    hours: hours > 0 ? hours : null,
    externalCostsMinor,
    effectiveHourlyMinor: final != null && hours > 0 ? Math.round((final - externalCostsMinor) / hours) : null,
    appliedMarginMicros: canSuggest ? strategyMargin : economicMargin,
    lines,
    issues: [...new Set(issues)],
    pricingTiers,
  };
}

export function emptyResult(issue: string): ServiceResult {
  return { status: "unconfigured", calculatedSubtotalMinor: null, suggestedSubtotalMinor: null, finalSubtotalMinor: null, effectiveSubtotalMinor: null, hasOverride: false, hours: null, externalCostsMinor: 0, effectiveHourlyMinor: null, appliedMarginMicros: null, lines: [], issues: [issue] };
}

export function createPricingSnapshot(input: EngineInput, result: ServiceResult): PricingSnapshot | null {
  const definition = input.pricing.definitions.find((item) => item.serviceType === input.serviceType);
  if (!definition) return null;
  const parameters = input.pricing.parameters.filter((item) => item.serviceDefinitionId === definition.id);
  return {
    schemaVersion: 1, createdAt: new Date().toISOString(), currency: input.currency,
    serviceType: input.serviceType, definition, parameters,
    options: input.pricing.options.filter((item) => parameters.some((parameter) => parameter.id === item.parameterId)),
    rules: input.pricing.rules.filter((item) => item.serviceDefinitionId === definition.id),
    economicProfile: input.pricing.economicProfiles.find((item) => item.currency === input.currency) ?? null,
    settings: { suggestionsEnabled: input.settings.suggestionsEnabled, suggestionStrategy: input.settings.suggestionStrategy, usdToArsMicros: input.settings.usdToArsMicros },
    parameterValues: input.parameterValues, result,
  };
}

export function resultFromService(service: QuoteService): ServiceResult | null {
  if (service.pricingSnapshotJson) {
    try {
      const snapshot = (JSON.parse(service.pricingSnapshotJson) as PricingSnapshot).result;
      // La investigación de mercado actualiza únicamente la columna de sugerido.
      // El snapshot de cálculo es histórico: nunca se reescribe desde mercado,
      // así que los campos persistidos son la fuente de verdad para los precios.
      const calculated = service.calculatedSubtotalMinor ?? snapshot.calculatedSubtotalMinor;
      const suggested = service.suggestedSubtotalMinor ?? snapshot.suggestedSubtotalMinor;
      const final = service.finalSubtotalMinor
        ?? service.manualSubtotalMinor
        ?? snapshot.finalSubtotalMinor
        ?? suggested
        ?? calculated;
      return {
        ...snapshot,
        status: final == null ? snapshot.status : "ready",
        calculatedSubtotalMinor: calculated,
        suggestedSubtotalMinor: suggested,
        finalSubtotalMinor: final,
        effectiveSubtotalMinor: final,
        hasOverride: service.hasOverride,
        effectiveHourlyMinor: final != null && snapshot.hours && snapshot.hours > 0
          ? Math.round((final - snapshot.externalCostsMinor) / snapshot.hours)
          : snapshot.effectiveHourlyMinor,
      };
    } catch { /* fallback below */ }
  }
  if (service.finalSubtotalMinor == null && service.calculatedSubtotalMinor == null) return null;
  const final = service.finalSubtotalMinor ?? service.manualSubtotalMinor ?? service.suggestedSubtotalMinor ?? service.calculatedSubtotalMinor;
  return { status: final == null ? "incomplete" : "ready", calculatedSubtotalMinor: service.calculatedSubtotalMinor, suggestedSubtotalMinor: service.suggestedSubtotalMinor, finalSubtotalMinor: final, effectiveSubtotalMinor: final, hasOverride: service.hasOverride, hours: null, externalCostsMinor: 0, effectiveHourlyMinor: null, appliedMarginMicros: null, lines: [], issues: [] };
}
