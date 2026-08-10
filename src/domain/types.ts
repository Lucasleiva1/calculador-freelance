export type Currency = "ARS" | "USD";
export type Theme = "warm" | "dark";
export type MarketScope = "argentina" | "international" | "both";
export type ServiceType = string;
export type PricingEngineType = "service" | "product" | "hybrid";
export type CalculatorKey = "professional-service-v1" | "physical-product-v1" | "hybrid-v1" | "unconfigured";
export type HelpMode = "guided" | "compact" | "off";
export type ClassificationOrigin = "automatic" | "ai_assisted" | "manual";
export type SaveStatus = "idle" | "saving" | "saved" | "error";
export type SuggestionStrategy = "competitive" | "balanced" | "premium";
export type ParameterType = "single_select" | "multi_select" | "boolean" | "number" | "duration" | "currency" | "percentage" | "text";
export type PricingRuleType = "fixed_amount" | "hours" | "per_unit" | "percentage" | "multiplier" | "external_cost";
export type AcquisitionMode = "auto_http" | "auto_browser" | "manual" | "disabled";
export type AutomationStatus = "APPROVED" | "UNREVIEWED" | "MANUAL_ONLY" | "BLOCKED";
export type MarketSourceStatus = "READY" | "FETCHING" | "SUCCESS" | "CACHED" | "MANUAL" | "BLOCKED" | "ERROR" | "DISABLED" | "NEEDS_CONFIGURATION";
export type MarketPriceType = "HOURLY" | "DAILY" | "PROJECT" | "PER_MINUTE" | "PER_ITEM" | "MONTHLY_SALARY" | "ANNUAL_SALARY" | "FIXED" | "RANGE" | "UNKNOWN";

export interface Client { id: string; name: string; company: string | null; email: string | null; whatsapp: string | null; country: string | null; notes: string | null; status: "active" | "archived"; createdAt: string; updatedAt: string; }
export interface ProjectSummary { id: string; clientId: string; clientName: string; name: string; currency: Currency; marketScope: MarketScope | null; status: "active" | "archived"; totalMinor: number | null; unpricedCount: number; updatedAt: string; }
export interface Quote { id: string; projectId: string; version: number; status: "draft" | "sent" | "accepted" | "rejected" | "archived"; currency: Currency; createdAt: string; updatedAt: string; }

export interface QuoteService {
  id: string; quoteId: string; serviceType: ServiceType; title: string; sortOrder: number;
  configurationVersion: number; configurationJson: string;
  calculatedSubtotalMinor: number | null; suggestedSubtotalMinor: number | null;
  finalSubtotalMinor: number | null; hasOverride: boolean;
  manualSubtotalMinor: number | null; manualReason: string | null;
  pricingSnapshotJson: string | null; serviceDefinitionVersion: number | null;
  rowRevision: number; deletedAt: string | null; createdAt: string; updatedAt: string;
}

export interface Preset { id: string; serviceType: ServiceType; name: string; origin: "system" | "user"; systemKey: string | null; configurationVersion: number; definitionVersion: number; configurationJson: string; createdAt: string; updatedAt: string; }

export interface AppSettings {
  theme: Theme; hourlyRateArsMinor: number | null; hourlyRateUsdMinor: number | null;
  usdToArsMicros: number | null; activeProjectId: string | null;
  suggestionsEnabled: boolean; suggestionStrategy: SuggestionStrategy; baseCurrency: Currency;
  helpMode: HelpMode; localAiEnabled: boolean; ollamaBaseUrl: string; ollamaModel: string | null;
  aiAutoApplyHighConfidence: boolean; updatedAt: string;
}

export interface ServiceDefinition {
  id: string; serviceType: ServiceType; name: string; description: string | null; version: number;
  enabled: boolean; suggestionsEnabled: boolean; defaultStrategy: SuggestionStrategy;
  competitiveMarginMicros: number | null; balancedMarginMicros: number | null; premiumMarginMicros: number | null;
  createdAt: string; updatedAt: string;
}

export interface ServiceParameter {
  id: string; serviceDefinitionId: string; parameterKey: string; name: string; label: string;
  parameterType: ParameterType; description: string | null; required: boolean; sortOrder: number;
  enabled: boolean; defaultValueJson: string | null; suggestionEnabled: boolean;
  isSystem: boolean; uiManaged: boolean; version: number; createdAt: string; updatedAt: string;
}

export interface ParameterOption { id: string; parameterId: string; label: string; value: string; sortOrder: number; enabled: boolean; createdAt: string; updatedAt: string; }
export interface PricingRule { id: string; serviceDefinitionId: string; parameterId: string | null; optionId: string | null; quantityParameterId: string | null; name: string; ruleType: PricingRuleType; numericValueMicros: number | null; amountArsMinor: number | null; amountUsdMinor: number | null; sortOrder: number; enabled: boolean; version: number; createdAt: string; updatedAt: string; }
export interface EconomicProfile { currency: Currency; monthlyIncomeTargetMinor: number | null; monthlyExpensesMinor: number | null; billableHoursMicros: number | null; reserveTaxMicros: number | null; desiredMarginMicros: number | null; defaultUrgencyMicros: number | null; workDays: number | null; vacationWeeks: number | null; manualHourlyRateMinor: number | null; updatedAt: string; }
export interface MarketSource {
  id: string; name: string; baseUrl: string | null; sourceType: string; regionsJson: string;
  supportedServicesJson: string; priority: number; enabled: boolean; usageMode: string;
  acquisitionMode: AcquisitionMode; cooldownHours: number | null; notes: string | null;
  isSystemSource: boolean; systemKey: string | null; defaultDataJson: string | null;
  purpose: string | null; dataContribution: string | null; appBenefit: string | null;
  participatesInSuggestions: boolean; automationStatus: AutomationStatus; currentStatus: MarketSourceStatus;
  adapterKey: string | null; lastRequestAt: string | null; lastSuccessAt: string | null;
  lastFailureAt: string | null; cooldownUntil: string | null; consecutiveFailures: number;
  lastHttpStatus: number | null; lastError: string | null; observationCount: number;
  archivedAt: string | null; businessSourceType: string; marketCountry: string | null;
  sourceCurrency: string | null; sourceUpdatedAt: string | null;
  classificationOrigin: ClassificationOrigin; classificationJson: string | null;
  createdAt: string; updatedAt: string;
}

export interface EngineCategory { id: string; parentId: string | null; slug: string; name: string; engineType: PricingEngineType | null; description: string | null; isSystem: boolean; sortOrder: number; createdAt: string; updatedAt: string; }
export interface PricingEngine { id: string; engineKey: string; name: string; description: string | null; engineType: PricingEngineType; categoryId: string | null; calculatorKey: CalculatorKey; serviceDefinitionId: string | null; unitKind: string; tagsJson: string; status: "draft" | "active" | "archived"; classificationOrigin: ClassificationOrigin; classificationConfidenceMicros: number | null; classificationExplanation: string | null; classificationVersion: number; isSystem: boolean; createdAt: string; updatedAt: string; archivedAt: string | null; }
export interface PricingEngineSource { engineId: string; sourceId: string; role: "reference" | "cost_input" | "context"; preference: "preferred" | "available" | "excluded"; participatesInSuggestions: boolean; matchScoreMicros: number; explanation: string | null; assignedBy: ClassificationOrigin; createdAt: string; updatedAt: string; }
export interface ClassificationInput { name: string; description?: string; deliverableKind?: "physical" | "digital" | "both" | "unknown"; activityKind?: "sale" | "service" | "both" | "unknown"; pricingUnit?: string; }
export interface ClassificationProposal { engineType: PricingEngineType; categoryId: string | null; categoryPath: string[]; calculatorKey: Exclude<CalculatorKey, "unconfigured">; businessActivity: string; pricingUnits: string[]; suggestedCostTypes: string[]; suggestedSourceTypes: string[]; tags: string[]; confidence: number; explanation: string; clarificationQuestion: string | null; aiAssisted: boolean; aiError: string | null; }
export interface SourceClassificationInput { name: string; baseUrl?: string; purpose?: string; dataContribution?: string; notes?: string; }
export interface SourceClassificationProposal { businessSourceType: string; suggestedEngineTypes: PricingEngineType[]; role: PricingEngineSource["role"]; tags: string[]; confidence: number; explanation: string; aiAssisted: boolean; aiError: string | null; }
export interface PricingEngineInput { id?: string; name: string; description?: string; engineType: PricingEngineType; categoryId: string | null; calculatorKey: CalculatorKey; unitKind: string; tags: string[]; status: "draft" | "active" | "archived"; classificationOrigin: ClassificationOrigin; classificationConfidence: number | null; classificationExplanation?: string; }
export interface EngineSourceInput { engineId: string; sourceId: string; role: PricingEngineSource["role"]; preference: PricingEngineSource["preference"]; participatesInSuggestions: boolean; matchScore: number; explanation?: string; assignedBy: ClassificationOrigin; }
export interface OllamaModel { name: string; parameterSize: string | null; quantizationLevel: string | null; size: number | null; }
export interface OllamaStatus { available: boolean; baseUrl: string; selectedModel: string | null; models: OllamaModel[]; message: string; }

export interface MarketObservation {
  id: string; sourceId: string; sourceName: string; origin: "AUTO" | "MANUAL";
  serviceType: string; subservice: string | null; category: string | null; region: string;
  country: string | null; currency: string; priceType: MarketPriceType; unit: string;
  priceMinMinor: number | null; priceMaxMinor: number | null; priceValueMinor: number | null;
  originalValueText: string; convertedValueMinor: number | null; convertedCurrency: string | null;
  exchangeRateMicros: number | null; exchangeRateDate: string | null; exchangeRateSource: string | null;
  experienceLevel: string | null; clientTier: string | null; sourceType: string; sourceUrl: string;
  publishedAt: string | null; retrievedAt: string; parserVersion: string; confidence: string;
  comparisonEligibility: "ELIGIBLE" | "CONTEXT_ONLY" | "REVIEW_REQUIRED" | "REJECTED" | "POSSIBLE_OUTLIER";
  exclusionReason: string | null; rawFingerprint: string; evidenceSnippet: string | null;
  notes: string | null; createdAt: string;
  snapshotIncluded: boolean | null; snapshotExclusionReason: string | null; snapshotNormalizedValueMinor: number | null;
}

export interface MarketSnapshot {
  id: string; quoteId: string | null; quoteServiceId: string | null; queryContextJson: string;
  currency: Currency; observationCount: number; comparableObservationCount: number; sourceCount: number;
  minimumFilteredMinor: number | null; p25Minor: number | null; marketMedianMinor: number | null;
  p75Minor: number | null; maximumFilteredMinor: number | null; confidenceLevel: "HIGH" | "MEDIUM" | "LOW" | "INSUFFICIENT";
  calculatedPriceMinor: number | null; suggestedPriceMinor: number | null; finalPriceMinorAtCreation: number | null;
  summaryJson: string; createdAt: string;
}

export interface MarketOverview { latestSnapshot: MarketSnapshot | null; observations: MarketObservation[]; history: MarketSnapshot[]; }
export interface MarketResearchJobItem { sourceId: string; sourceName: string; status: MarketSourceStatus | "READY"; message: string | null; observationCount: number; }
export interface MarketResearchJob { id: string; quoteServiceId: string; status: "RUNNING" | "COMPLETED" | "CANCELLED" | "ERROR"; completed: number; total: number; cancelRequested: boolean; items: MarketResearchJobItem[]; snapshotId: string | null; error: string | null; startedAt: string; finishedAt: string | null; }
export interface MarketObservationPreview { serviceType: string; subservice: string | null; priceMinMinor: number | null; priceMaxMinor: number | null; priceValueMinor: number | null; currency: string; unit: string; priceType: MarketPriceType; region: string; evidence: string | null; }
export interface SourceTestResult { sourceId: string; status: MarketSourceStatus; message: string; httpStatus: number | null; observations: MarketObservationPreview[]; }
export interface PricingConfiguration { definitions: ServiceDefinition[]; parameters: ServiceParameter[]; options: ParameterOption[]; rules: PricingRule[]; economicProfiles: EconomicProfile[]; marketSources: MarketSource[]; engineCategories: EngineCategory[]; pricingEngines: PricingEngine[]; engineSources: PricingEngineSource[]; }

export interface Bootstrap { clients: Client[]; projects: ProjectSummary[]; presets: Preset[]; settings: AppSettings; pricing: PricingConfiguration; }
export interface Workspace { project: ProjectSummary; quote: Quote; services: QuoteService[]; }
export interface ClientInput { id?: string; name: string; company?: string; email?: string; whatsapp?: string; country?: string; notes?: string; }
export interface CreateProjectInput { name: string; clientId?: string; newClient?: ClientInput; currency: Currency; marketScope: MarketScope; }
export interface SettingsInput { theme: Theme; hourlyRateArsMinor: number | null; hourlyRateUsdMinor: number | null; usdToArsMicros: number | null; suggestionsEnabled: boolean; suggestionStrategy: SuggestionStrategy; baseCurrency: Currency; helpMode: HelpMode; localAiEnabled: boolean; ollamaBaseUrl: string; ollamaModel: string | null; aiAutoApplyHighConfidence: boolean; }
export interface SaveServiceInput { id: string; title: string; configurationVersion: number; configurationJson: string; calculatedSubtotalMinor: number | null; suggestedSubtotalMinor: number | null; finalSubtotalMinor: number | null; hasOverride: boolean; manualSubtotalMinor: number | null; manualReason: string | null; pricingSnapshotJson: string | null; serviceDefinitionVersion: number | null; expectedRevision: number; }
export interface PresetInput { id?: string; serviceType: ServiceType; name: string; configurationVersion: number; definitionVersion?: number; configurationJson: string; }

export type ServiceDefinitionInput = Pick<ServiceDefinition, "id" | "name" | "description" | "enabled" | "suggestionsEnabled" | "defaultStrategy" | "competitiveMarginMicros" | "balancedMarginMicros" | "premiumMarginMicros">;
export interface ServiceParameterInput { id?: string; serviceDefinitionId: string; parameterKey: string; name: string; label: string; parameterType: ParameterType; description?: string; required: boolean; sortOrder: number; enabled: boolean; defaultValueJson: string | null; suggestionEnabled: boolean; }
export interface ParameterOptionInput { id?: string; parameterId: string; label: string; value: string; sortOrder: number; enabled: boolean; }
export interface PricingRuleInput { id?: string; serviceDefinitionId: string; parameterId: string | null; optionId: string | null; quantityParameterId: string | null; name: string; ruleType: PricingRuleType; numericValueMicros: number | null; amountArsMinor: number | null; amountUsdMinor: number | null; sortOrder: number; enabled: boolean; }
export type EconomicProfileInput = Omit<EconomicProfile, "updatedAt">;
export interface MarketSourceInput { id?: string; name: string; baseUrl?: string; sourceType: string; regionsJson: string; supportedServicesJson: string; priority: number; enabled: boolean; usageMode: string; acquisitionMode: AcquisitionMode; cooldownHours: number | null; notes?: string; purpose?: string; dataContribution?: string; appBenefit?: string; participatesInSuggestions: boolean; businessSourceType?: string; marketCountry?: string; sourceCurrency?: string; sourceUpdatedAt?: string; classificationOrigin?: ClassificationOrigin; classificationJson?: string; }
export interface ManualObservationInput { sourceId: string; serviceType: string; subservice?: string; category?: string; region: string; country?: string; currency: string; priceType: MarketPriceType; unit: string; priceMinMinor: number | null; priceMaxMinor: number | null; priceValueMinor: number | null; experienceLevel?: string; clientTier?: string; publishedAt?: string; sourceUrl: string; notes?: string; }
export interface MarketObservationFilter { serviceType?: string; region?: string; sourceId?: string; priceType?: MarketPriceType; currency?: string; query?: string; }

export interface PricingContext { currency: Currency; hourlyRateMinor: number | null; usdToArsMicros: number | null; }
export interface PriceLine { id?: string; label: string; kind?: "base" | PricingRuleType | "external" | "margin" | "override"; amountMinor: number; detail?: string; }
export interface ServiceResult {
  status: "ready" | "incomplete" | "unconfigured";
  calculatedSubtotalMinor: number | null; suggestedSubtotalMinor: number | null;
  finalSubtotalMinor: number | null; effectiveSubtotalMinor: number | null; hasOverride: boolean;
  hours: number | null; externalCostsMinor: number; effectiveHourlyMinor: number | null;
  appliedMarginMicros: number | null; lines: PriceLine[]; issues: string[];
  engineKind?: PricingEngineType;
  pricingTiers?: ProductPricingTiers;
  productMetrics?: ProductMetrics;
}
export interface ProductPriceTier { unitMinor: number; totalMinor: number; marginMicros: number; }
export interface ProductPricingTiers { floor: ProductPriceTier; recommended: ProductPriceTier; premium: ProductPriceTier; selected: "floor" | "recommended" | "premium"; }
export interface ProductMetrics { quantity: number; costUnitMinor: number; productionCostMinor: number; revenueMinor: number; grossProfitMinor: number; marginMicros: number; markupMicros: number; sellingFeesMinor: number; }
export interface PricingSnapshot { schemaVersion: 1; createdAt: string; currency: Currency; serviceType: ServiceType; definition: ServiceDefinition; parameters: ServiceParameter[]; options: ParameterOption[]; rules: PricingRule[]; economicProfile: EconomicProfile | null; settings: Pick<AppSettings, "suggestionsEnabled" | "suggestionStrategy" | "usdToArsMicros">; parameterValues: Record<string, unknown>; result: ServiceResult; }
export interface ServiceConfigurationEnvelope<T> { schemaVersion: number; serviceType: ServiceType; data: T; }
export interface ServiceModuleDefinition<T> { type: ServiceType; label: string; schemaVersion: number; createDefaultConfiguration: () => T; validate: (configuration: T) => string[]; calculate: (configuration: T, context: PricingContext, manualSubtotalMinor?: number | null) => ServiceResult; summarize: (configuration: T) => string[]; }
