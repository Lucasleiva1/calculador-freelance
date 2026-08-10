import { describe, expect, it } from "vitest";
import { applySuggestedDefaults, calculateSustainableRate, createPricingSnapshot, runPricingEngine } from "./pricingEngine";
import type { AppSettings, EconomicProfile, ParameterOption, PricingConfiguration, PricingRule, ServiceDefinition, ServiceParameter } from "./types";

const stamp = "2026-08-09T00:00:00Z";
const definition: ServiceDefinition = { id: "service-programming", serviceType: "programming", name: "Programación", description: null, version: 4, enabled: true, suggestionsEnabled: true, defaultStrategy: "balanced", competitiveMarginMicros: 100_000, balancedMarginMicros: 200_000, premiumMarginMicros: 250_000, createdAt: stamp, updatedAt: stamp };
const parameter = (id: string, key: string, type: ServiceParameter["parameterType"] = "number"): ServiceParameter => ({ id, serviceDefinitionId: definition.id, parameterKey: key, name: key, label: key, parameterType: type, description: null, required: false, sortOrder: 1, enabled: true, defaultValueJson: null, suggestionEnabled: false, isSystem: false, uiManaged: false, version: 1, createdAt: stamp, updatedAt: stamp });
const hours = parameter("hours", "estimatedHours");
const quantity = parameter("quantity", "quantity");
const plan = parameter("plan", "plan", "single_select");
const option: ParameterOption = { id: "premium", parameterId: plan.id, label: "Premium", value: "premium", sortOrder: 1, enabled: true, createdAt: stamp, updatedAt: stamp };
const rule = (id: string, ruleType: PricingRule["ruleType"], partial: Partial<PricingRule> = {}): PricingRule => ({ id, serviceDefinitionId: definition.id, parameterId: plan.id, optionId: option.id, quantityParameterId: null, name: id, ruleType, numericValueMicros: null, amountArsMinor: null, amountUsdMinor: null, sortOrder: 1, enabled: true, version: 1, createdAt: stamp, updatedAt: stamp, ...partial });
const profile: EconomicProfile = { currency: "USD", monthlyIncomeTargetMinor: null, monthlyExpensesMinor: null, billableHoursMicros: null, reserveTaxMicros: null, desiredMarginMicros: 200_000, defaultUrgencyMicros: null, workDays: null, vacationWeeks: null, manualHourlyRateMinor: 10_000, updatedAt: stamp };
const settings: AppSettings = { theme: "warm", hourlyRateArsMinor: null, hourlyRateUsdMinor: 10_000, usdToArsMicros: null, activeProjectId: null, suggestionsEnabled: true, suggestionStrategy: "premium", baseCurrency: "USD", helpMode: "guided", localAiEnabled: false, ollamaBaseUrl: "http://127.0.0.1:11434", ollamaModel: null, aiAutoApplyHighConfidence: false, updatedAt: stamp };
const rules: PricingRule[] = [
  rule("fijo", "fixed_amount", { amountUsdMinor: 1_000, sortOrder: 1 }),
  rule("horas", "hours", { numericValueMicros: 1_000_000, sortOrder: 2 }),
  rule("unidad", "per_unit", { amountUsdMinor: 200, numericValueMicros: 1_000_000, quantityParameterId: quantity.id, sortOrder: 3 }),
  rule("recargo", "percentage", { numericValueMicros: 100_000, sortOrder: 4 }),
  rule("multiplicador", "multiplier", { numericValueMicros: 1_500_000, sortOrder: 5 }),
  rule("tercero", "external_cost", { amountUsdMinor: 500, sortOrder: 6 }),
];
const pricing: PricingConfiguration = { definitions: [definition], parameters: [hours, quantity, plan], options: [option], rules, economicProfiles: [profile], marketSources: [], engineCategories: [], pricingEngines: [], engineSources: [] };
const input = { serviceType: "programming" as const, currency: "USD" as const, parameterValues: { estimatedHours: 2, quantity: 3, plan: "premium" }, settings, pricing };

describe("PricingEngine configurable", () => {
  it("aplica importes, horas, unidad, recargo, multiplicador, costo externo y margen explicable", () => {
    const result = runPricingEngine(input);
    expect(result.hours).toBe(3);
    expect(result.lines.map((line) => line.kind)).toEqual(["base", "fixed_amount", "hours", "per_unit", "percentage", "multiplier", "external_cost", "margin"]);
    expect(result.calculatedSubtotalMinor).toBe(65_800);
    expect(result.suggestedSubtotalMinor).toBe(70_187);
    expect(result.finalSubtotalMinor).toBe(70_187);
  });

  it("distingue recargo de margen y respeta el override como precio final autoritativo", () => {
    const result = runPricingEngine({ ...input, finalOverrideMinor: 80_000, hasOverride: true });
    expect(result.calculatedSubtotalMinor).toBe(65_800);
    expect(result.finalSubtotalMinor).toBe(80_000);
    expect(result.hasOverride).toBe(true);
    expect(result.lines.at(-1)?.kind).toBe("override");
  });

  it("apaga sugerencias sin borrar ni inventar el precio calculado", () => {
    const result = runPricingEngine({ ...input, settings: { ...settings, suggestionsEnabled: false } });
    expect(result.suggestedSubtotalMinor).toBeNull();
    expect(result.finalSubtotalMinor).toBe(result.calculatedSubtotalMinor);
    expect(applySuggestedDefaults({}, [{ ...hours, suggestionEnabled: true, defaultValueJson: "8" }], false)).toEqual({});
  });

  it("conserva en el snapshot el resultado y la versión originales", () => {
    const result = runPricingEngine(input);
    const snapshot = createPricingSnapshot(input, result)!;
    const persisted = JSON.parse(JSON.stringify(snapshot)) as typeof snapshot;
    definition.version = 99;
    rules[0].amountUsdMinor = 99_999;
    expect(persisted.definition.version).toBe(4);
    expect(persisted.result.finalSubtotalMinor).toBe(70_187);
  });
});

describe("economía sostenible", () => {
  it("incluye gastos y reserva con gross-up antes de dividir por horas", () => {
    const result = calculateSustainableRate({ ...profile, monthlyIncomeTargetMinor: 100_000, monthlyExpensesMinor: 20_000, billableHoursMicros: 100_000_000, reserveTaxMicros: 200_000 });
    expect(result.monthlyRequiredMinor).toBe(150_000);
    expect(result.rateMinor).toBe(1_500);
  });
});
