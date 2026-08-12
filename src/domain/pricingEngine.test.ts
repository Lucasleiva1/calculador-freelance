import { describe, expect, it } from "vitest";
import { applySuggestedDefaults, calculateSustainableRate, createPricingSnapshot, resultFromService, runPricingEngine } from "./pricingEngine";
import type { AppSettings, EconomicProfile, ParameterOption, PricingConfiguration, PricingEngine, PricingRule, QuoteService, ServiceDefinition, ServiceParameter } from "./types";

const stamp = "2026-08-09T00:00:00Z";
const definition: ServiceDefinition = { id: "service-programming", serviceType: "programming", name: "Programación", description: null, version: 4, enabled: true, suggestionsEnabled: true, defaultStrategy: "balanced", competitiveMarginMicros: 100_000, balancedMarginMicros: 200_000, premiumMarginMicros: 250_000, createdAt: stamp, updatedAt: stamp };
const parameter = (id: string, key: string, type: ServiceParameter["parameterType"] = "number"): ServiceParameter => ({ id, serviceDefinitionId: definition.id, parameterKey: key, name: key, label: key, parameterType: type, description: null, required: false, sortOrder: 1, enabled: true, defaultValueJson: null, suggestionEnabled: false, isSystem: false, uiManaged: false, version: 1, createdAt: stamp, updatedAt: stamp });
const hours = parameter("hours", "estimatedHours");
const quantity = parameter("quantity", "quantity");
const plan = parameter("plan", "plan", "single_select");
const option: ParameterOption = { id: "premium", parameterId: plan.id, label: "Premium", value: "premium", sortOrder: 1, enabled: true, createdAt: stamp, updatedAt: stamp };
const rule = (id: string, ruleType: PricingRule["ruleType"], partial: Partial<PricingRule> = {}): PricingRule => ({ id, serviceDefinitionId: definition.id, parameterId: plan.id, optionId: option.id, quantityParameterId: null, name: id, ruleType, numericValueMicros: null, amountArsMinor: null, amountUsdMinor: null, sortOrder: 1, enabled: true, version: 1, createdAt: stamp, updatedAt: stamp, ...partial });
const engine: PricingEngine = { id: "engine-programming", engineKey: "programming", name: "Programación", description: null, engineType: "service", categoryId: null, calculatorKey: "professional-service-v1", serviceDefinitionId: definition.id, unitKind: "hour", tagsJson: "[]", status: "active", classificationOrigin: "automatic", classificationConfidenceMicros: null, classificationExplanation: null, classificationVersion: 1, isSystem: true, createdAt: stamp, updatedAt: stamp, archivedAt: null };
const profile: EconomicProfile = { engineId: engine.id, currency: "USD", monthlyIncomeTargetMinor: null, monthlyExpensesMinor: null, billableHoursMicros: null, reserveTaxMicros: null, desiredMarginMicros: 200_000, defaultUrgencyMicros: null, workDays: null, vacationWeeks: null, manualHourlyRateMinor: 10_000, updatedAt: stamp };
const settings: AppSettings = { theme: "warm", hourlyRateArsMinor: null, hourlyRateUsdMinor: 10_000, usdToArsMicros: null, activeProjectId: null, suggestionsEnabled: true, suggestionStrategy: "premium", baseCurrency: "USD", helpMode: "guided", localAiEnabled: false, ollamaBaseUrl: "http://127.0.0.1:11434", ollamaModel: null, aiAutoApplyHighConfidence: false, updatedAt: stamp };
const rules: PricingRule[] = [
  rule("fijo", "fixed_amount", { amountUsdMinor: 1_000, sortOrder: 1 }),
  rule("horas", "hours", { numericValueMicros: 1_000_000, sortOrder: 2 }),
  rule("unidad", "per_unit", { amountUsdMinor: 200, numericValueMicros: 1_000_000, quantityParameterId: quantity.id, sortOrder: 3 }),
  rule("recargo", "percentage", { numericValueMicros: 100_000, sortOrder: 4 }),
  rule("multiplicador", "multiplier", { numericValueMicros: 1_500_000, sortOrder: 5 }),
  rule("tercero", "external_cost", { amountUsdMinor: 500, sortOrder: 6 }),
];
const pricing: PricingConfiguration = { definitions: [definition], parameters: [hours, quantity, plan], options: [option], rules, economicProfiles: [profile], marketSources: [], engineCategories: [], pricingEngines: [engine], engineSources: [] };
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
  it("muestra una sugerencia de mercado persistida sin reescribir el snapshot ni el precio final", () => {
    const result = runPricingEngine(input);
    const snapshot = JSON.stringify(createPricingSnapshot(input, result));
    const service: QuoteService = {
      id: "service", quoteId: "quote", serviceType: "programming", title: "Sitio", sortOrder: 0,
      configurationVersion: 1, configurationJson: JSON.stringify({ data: input.parameterValues }),
      calculatedSubtotalMinor: result.calculatedSubtotalMinor, suggestedSubtotalMinor: 88_000,
      finalSubtotalMinor: result.finalSubtotalMinor, hasOverride: false, manualSubtotalMinor: null,
      manualReason: null, pricingSnapshotJson: snapshot, serviceDefinitionVersion: definition.version,
      rowRevision: 4, deletedAt: null, createdAt: stamp, updatedAt: stamp,
    };
    const visible = resultFromService(service)!;
    expect(visible.suggestedSubtotalMinor).toBe(88_000);
    expect(visible.finalSubtotalMinor).toBe(result.finalSubtotalMinor);
    expect(service.pricingSnapshotJson).toBe(snapshot);
  });
});

describe("economía sostenible", () => {
  it("incluye gastos y reserva con gross-up antes de dividir por horas", () => {
    const result = calculateSustainableRate({ ...profile, monthlyIncomeTargetMinor: 100_000, monthlyExpensesMinor: 20_000, billableHoursMicros: 100_000_000, reserveTaxMicros: 200_000 });
    expect(result.monthlyRequiredMinor).toBe(150_000);
    expect(result.rateMinor).toBe(1_500);
  });

  it("no hereda la tarifa de programación en Diseño de estampas", () => {
    const printDefinition: ServiceDefinition = { ...definition, id: "service-print-design", serviceType: "print-design", name: "Diseño de estampas" };
    const printHours = { ...hours, id: "print-hours", serviceDefinitionId: printDefinition.id };
    const printEngine: PricingEngine = { ...engine, id: "engine-print-design", engineKey: "print-design", name: "Diseño de estampas", serviceDefinitionId: printDefinition.id };
    const isolatedPricing: PricingConfiguration = { ...pricing, definitions: [definition, printDefinition], parameters: [hours, printHours], options: [], rules: [], pricingEngines: [engine, printEngine], economicProfiles: [profile] };
    const missing = runPricingEngine({ serviceType: "print-design", currency: "USD", parameterValues: { estimatedHours: 2.5 }, settings, pricing: isolatedPricing });
    expect(missing.status).toBe("incomplete");
    expect(missing.calculatedSubtotalMinor).toBeNull();
    expect(missing.issues).toContain("Configurá tu economía en USD.");

    const ownProfile: EconomicProfile = { ...profile, engineId: printEngine.id, desiredMarginMicros: null, manualHourlyRateMinor: 2_500 };
    const parameterValues = { hasReference: true, materialType: "ready", clientTier: "small", productType: "shirt", garmentTone: "light", printSystem: "design-only", workTasks: ["optimize-image"], complexity: "basic", estimatedHours: 2.5 };
    const ready = runPricingEngine({ serviceType: "print-design", currency: "USD", parameterValues, settings, pricing: { ...isolatedPricing, economicProfiles: [profile, ownProfile] } });
    expect(ready.calculatedSubtotalMinor).toBe(6_250);
    expect(ready.finalSubtotalMinor).toBeNull();
    expect(ready.status).toBe("ready");
  });

  it("aplica DTF sólo al núcleo, agrega extras después y deja externos fuera del margen", () => {
    const printDefinition: ServiceDefinition = { ...definition, id: "service-print-design", serviceType: "print-design", name: "Diseño de estampas" };
    const printEngine: PricingEngine = { ...engine, id: "engine-print-design", engineKey: "print-design", name: "Diseño de estampas", serviceDefinitionId: printDefinition.id };
    const printProfile: EconomicProfile = { ...profile, engineId: printEngine.id, desiredMarginMicros: 200_000, manualHourlyRateMinor: 2_500 };
    const printPricing: PricingConfiguration = { ...pricing, definitions: [printDefinition], parameters: [], options: [], rules: [], pricingEngines: [printEngine], economicProfiles: [printProfile] };
    const baseValues = { hasReference: true, materialType: "ready", clientTier: "medium", productType: "shirt", garmentTone: "dark", printSystem: "dtf", workTasks: ["optimize-image"], complexity: "basic", estimatedHours: 2, deliveryExtras: ["psd"] };
    const result = runPricingEngine({ serviceType: "print-design", currency: "USD", parameterValues: baseValues, externalCosts: [{ id: "printing", name: "Impresión tercerizada", amountMinor: 1_000, currency: "USD", note: "" }], settings, pricing: printPricing });
    // Núcleo 5.000 + DTF 750 + extra 1.250 = 7.000; margen 20% = 8.750; externo = 1.000.
    expect(result.calculatedSubtotalMinor).toBe(9_750);
    expect(result.lines.find((line) => line.label === "Preparación para DTF")?.amountMinor).toBe(750);
    expect(result.lines.find((line) => line.label === "Entregables adicionales")?.amountMinor).toBe(1_250);
    expect(result.finalSubtotalMinor).toBeNull();
  });

  it("distingue sublimación A4 de sublimación dividida", () => {
    const printDefinition: ServiceDefinition = { ...definition, id: "service-print-design", serviceType: "print-design", name: "Diseño de estampas" };
    const printEngine: PricingEngine = { ...engine, id: "engine-print-design", engineKey: "print-design", name: "Diseño de estampas", serviceDefinitionId: printDefinition.id };
    const printProfile: EconomicProfile = { ...profile, engineId: printEngine.id, desiredMarginMicros: null, manualHourlyRateMinor: 2_500 };
    const printPricing: PricingConfiguration = { ...pricing, definitions: [printDefinition], parameters: [], options: [], rules: [], pricingEngines: [printEngine], economicProfiles: [printProfile] };
    const values = { hasReference: true, materialType: "ready", clientTier: "small", productType: "shirt", garmentTone: "light", printSystem: "sublimation", workTasks: ["optimize-image"], complexity: "basic", estimatedHours: 2 };
    const a4 = runPricingEngine({ serviceType: "print-design", currency: "USD", parameterValues: { ...values, sublimationFitsA4: true }, settings, pricing: printPricing });
    const divided = runPricingEngine({ serviceType: "print-design", currency: "USD", parameterValues: { ...values, sublimationFitsA4: false }, settings, pricing: printPricing });
    expect(a4.calculatedSubtotalMinor).toBe(5_000);
    expect(divided.calculatedSubtotalMinor).toBe(5_750);
  });

  it("no convierte el sostenible ni el mercado en precio final sin una elección", () => {
    const snapshotResult = { ...runPricingEngine(input), finalSubtotalMinor: 99_000, effectiveSubtotalMinor: 99_000 };
    const printService: QuoteService = {
      id: "print", quoteId: "quote", serviceType: "print-design", title: "Estampa", sortOrder: 0,
      configurationVersion: 3, configurationJson: "{}", calculatedSubtotalMinor: 50_000,
      suggestedSubtotalMinor: 70_000, finalSubtotalMinor: null, hasOverride: false,
      manualSubtotalMinor: null, manualReason: null,
      pricingSnapshotJson: JSON.stringify({ ...createPricingSnapshot(input, snapshotResult), result: snapshotResult }),
      serviceDefinitionVersion: 2, rowRevision: 0, deletedAt: null, createdAt: stamp, updatedAt: stamp,
    };
    const visible = resultFromService(printService)!;
    expect(visible.calculatedSubtotalMinor).toBe(50_000);
    expect(visible.suggestedSubtotalMinor).toBe(70_000);
    expect(visible.finalSubtotalMinor).toBeNull();
  });
});
