export type Currency = "ARS" | "USD";
export type Theme = "warm" | "dark";
export type MarketScope = "argentina" | "international" | "both";
export type ServiceType = "video-editing" | "programming";
export type SaveStatus = "idle" | "saving" | "saved" | "error";
export type SuggestionStrategy = "competitive" | "balanced" | "premium";
export type ParameterType = "single_select" | "multi_select" | "boolean" | "number" | "duration" | "currency" | "percentage" | "text";
export type PricingRuleType = "fixed_amount" | "hours" | "per_unit" | "percentage" | "multiplier" | "external_cost";

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
  suggestionsEnabled: boolean; suggestionStrategy: SuggestionStrategy; baseCurrency: Currency; updatedAt: string;
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
export interface MarketSource { id: string; name: string; baseUrl: string | null; sourceType: string; regionsJson: string; supportedServicesJson: string; priority: number; enabled: boolean; usageMode: string; acquisitionMode: "auto_http" | "auto_browser" | "manual" | "disabled"; cooldownHours: number | null; notes: string | null; isSystemSource: boolean; systemKey: string | null; defaultDataJson: string | null; createdAt: string; updatedAt: string; }
export interface PricingConfiguration { definitions: ServiceDefinition[]; parameters: ServiceParameter[]; options: ParameterOption[]; rules: PricingRule[]; economicProfiles: EconomicProfile[]; marketSources: MarketSource[]; }

export interface Bootstrap { clients: Client[]; projects: ProjectSummary[]; presets: Preset[]; settings: AppSettings; pricing: PricingConfiguration; }
export interface Workspace { project: ProjectSummary; quote: Quote; services: QuoteService[]; }
export interface ClientInput { id?: string; name: string; company?: string; email?: string; whatsapp?: string; country?: string; notes?: string; }
export interface CreateProjectInput { name: string; clientId?: string; newClient?: ClientInput; currency: Currency; marketScope: MarketScope; }
export interface SettingsInput { theme: Theme; hourlyRateArsMinor: number | null; hourlyRateUsdMinor: number | null; usdToArsMicros: number | null; suggestionsEnabled: boolean; suggestionStrategy: SuggestionStrategy; baseCurrency: Currency; }
export interface SaveServiceInput { id: string; title: string; configurationVersion: number; configurationJson: string; calculatedSubtotalMinor: number | null; suggestedSubtotalMinor: number | null; finalSubtotalMinor: number | null; hasOverride: boolean; manualSubtotalMinor: number | null; manualReason: string | null; pricingSnapshotJson: string | null; serviceDefinitionVersion: number | null; expectedRevision: number; }
export interface PresetInput { id?: string; serviceType: ServiceType; name: string; configurationVersion: number; definitionVersion?: number; configurationJson: string; }

export type ServiceDefinitionInput = Pick<ServiceDefinition, "id" | "name" | "description" | "enabled" | "suggestionsEnabled" | "defaultStrategy" | "competitiveMarginMicros" | "balancedMarginMicros" | "premiumMarginMicros">;
export interface ServiceParameterInput { id?: string; serviceDefinitionId: string; parameterKey: string; name: string; label: string; parameterType: ParameterType; description?: string; required: boolean; sortOrder: number; enabled: boolean; defaultValueJson: string | null; suggestionEnabled: boolean; }
export interface ParameterOptionInput { id?: string; parameterId: string; label: string; value: string; sortOrder: number; enabled: boolean; }
export interface PricingRuleInput { id?: string; serviceDefinitionId: string; parameterId: string | null; optionId: string | null; quantityParameterId: string | null; name: string; ruleType: PricingRuleType; numericValueMicros: number | null; amountArsMinor: number | null; amountUsdMinor: number | null; sortOrder: number; enabled: boolean; }
export type EconomicProfileInput = Omit<EconomicProfile, "updatedAt">;
export interface MarketSourceInput { id?: string; name: string; baseUrl?: string; sourceType: string; regionsJson: string; supportedServicesJson: string; priority: number; enabled: boolean; usageMode: string; acquisitionMode: MarketSource["acquisitionMode"]; cooldownHours: number | null; notes?: string; }

export interface PricingContext { currency: Currency; hourlyRateMinor: number | null; usdToArsMicros: number | null; }
export interface PriceLine { id?: string; label: string; kind?: "base" | PricingRuleType | "external" | "margin" | "override"; amountMinor: number; detail?: string; }
export interface ServiceResult {
  status: "ready" | "incomplete" | "unconfigured";
  calculatedSubtotalMinor: number | null; suggestedSubtotalMinor: number | null;
  finalSubtotalMinor: number | null; effectiveSubtotalMinor: number | null; hasOverride: boolean;
  hours: number | null; externalCostsMinor: number; effectiveHourlyMinor: number | null;
  appliedMarginMicros: number | null; lines: PriceLine[]; issues: string[];
}
export interface PricingSnapshot { schemaVersion: 1; createdAt: string; currency: Currency; serviceType: ServiceType; definition: ServiceDefinition; parameters: ServiceParameter[]; options: ParameterOption[]; rules: PricingRule[]; economicProfile: EconomicProfile | null; settings: Pick<AppSettings, "suggestionsEnabled" | "suggestionStrategy" | "usdToArsMicros">; parameterValues: Record<string, unknown>; result: ServiceResult; }
export interface ServiceConfigurationEnvelope<T> { schemaVersion: number; serviceType: ServiceType; data: T; }
export interface ServiceModuleDefinition<T> { type: ServiceType; label: string; schemaVersion: number; createDefaultConfiguration: () => T; validate: (configuration: T) => string[]; calculate: (configuration: T, context: PricingContext, manualSubtotalMinor?: number | null) => ServiceResult; summarize: (configuration: T) => string[]; }
