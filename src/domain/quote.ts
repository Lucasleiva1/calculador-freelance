import type { AppSettings, PricingConfiguration, QuoteService, ServiceResult, Workspace } from "./types";
import { parseProgrammingEnvelope } from "./programming";
import { parseProfessionalEnvelope } from "./professional";
import { parseVideoEnvelope } from "./video";
import type { HybridConfiguration, ProductConfiguration } from "./product";
import { calculateHybrid, calculateProduct } from "./product";
import { activeHourlyRate, economicProfileFor, emptyResult, resultFromService, runPricingEngine } from "./pricingEngine";
import type { ServiceConfigurationEnvelope } from "./types";
import type { PrintDesignPriceSelection } from "./printDesign";

export interface EvaluatedService { service: QuoteService; result: ServiceResult; }
export interface ProjectResult {
  services: EvaluatedService[]; totalMinor: number | null; totalHours: number;
  externalCostsMinor: number; effectiveHourlyMinor: number | null; marginMicros: number | null;
  pricingTiers: { floorMinor: number | null; recommendedMinor: number | null; premiumMinor: number | null };
  unpricedCount: number; isPartial: boolean;
}

function liveResult(service: QuoteService, workspace: Workspace, settings: AppSettings, pricing: PricingConfiguration) {
  try {
    if (service.serviceType === "video-editing") {
      const config = parseVideoEnvelope(service.configurationJson).data;
      return runPricingEngine({ serviceType: service.serviceType, currency: workspace.quote.currency, parameterValues: config as unknown as Record<string, unknown>, externalCosts: config.externalCosts, fixedUrgencyMinor: config.urgencyFeeMinor, finalOverrideMinor: service.finalSubtotalMinor ?? service.manualSubtotalMinor, hasOverride: service.hasOverride || service.manualSubtotalMinor != null, settings, pricing });
    }
    const engine = pricing.pricingEngines.find((item) => item.engineKey === service.serviceType);
    if (engine?.calculatorKey === "physical-product-v1" || engine?.calculatorKey === "hybrid-v1") {
      const config = (JSON.parse(service.configurationJson) as ServiceConfigurationEnvelope<ProductConfiguration | HybridConfiguration>).data;
      const profile = economicProfileFor(pricing, service.serviceType, workspace.quote.currency);
      const context = { currency: workspace.quote.currency, hourlyRateMinor: activeHourlyRate(profile), usdToArsMicros: settings.usdToArsMicros };
      return engine.calculatorKey === "hybrid-v1" ? calculateHybrid(config as HybridConfiguration, context) : calculateProduct(config, context);
    }
    const config = service.serviceType === "programming"
      ? parseProgrammingEnvelope(service.configurationJson).data
      : parseProfessionalEnvelope(service.configurationJson, service.serviceType).data;
    const printSelection = service.serviceType === "print-design" ? config.parameterValues.priceSelection as PrintDesignPriceSelection | undefined : undefined;
    return runPricingEngine({ serviceType: service.serviceType, currency: workspace.quote.currency, parameterValues: config.parameterValues, externalCosts: config.externalCosts, finalOverrideMinor: service.serviceType === "print-design" ? printSelection?.amountMinor ?? null : service.finalSubtotalMinor ?? service.manualSubtotalMinor, hasOverride: service.serviceType === "print-design" ? Boolean(printSelection) : service.hasOverride || service.manualSubtotalMinor != null, settings, pricing });
  } catch { return emptyResult("La configuración guardada no se pudo leer."); }
}

function manualEconomyIsMissing(service: QuoteService, workspace: Workspace, pricing: PricingConfiguration) {
  const engine = pricing.pricingEngines.find((item) => item.engineKey === service.serviceType && item.status === "active");
  if (!engine || !["professional-service-v1", "hybrid-v1"].includes(engine.calculatorKey)) return false;
  return economicProfileFor(pricing, service.serviceType, workspace.quote.currency) == null;
}

export function evaluateWorkspace(workspace: Workspace, settings: AppSettings, pricing: PricingConfiguration): ProjectResult {
  const services = workspace.services.map((service): EvaluatedService => ({
    service,
    result: manualEconomyIsMissing(service, workspace, pricing)
      ? liveResult(service, workspace, settings, pricing)
      : resultFromService(service) ?? liveResult(service, workspace, settings, pricing),
  }));
  const priced = services.filter(({ result }) => result.finalSubtotalMinor != null);
  const unpricedCount = services.length - priced.length;
  const totalMinor = priced.length === 0 ? null : priced.reduce((sum, item) => sum + (item.result.finalSubtotalMinor ?? 0), 0);
  const totalHours = services.reduce((sum, item) => sum + (item.result.hours ?? 0), 0);
  const externalCostsMinor = services.reduce((sum, item) => sum + item.result.externalCostsMinor, 0);
  const tierTotal = (tier: "floor" | "recommended" | "premium") => {
    const values = priced.map(({ result }) => result.pricingTiers?.[tier].totalMinor
      ?? (tier === "floor" ? result.calculatedSubtotalMinor : tier === "recommended" ? result.suggestedSubtotalMinor ?? result.calculatedSubtotalMinor : result.suggestedSubtotalMinor ?? result.calculatedSubtotalMinor));
    return values.length === 0 || values.some((value) => value == null) ? null : values.reduce<number>((sum, value) => sum + (value ?? 0), 0);
  };
  const weightedMarginBase = priced.reduce((sum, item) => sum + (item.result.finalSubtotalMinor ?? 0), 0);
  const marginMicros = weightedMarginBase > 0 ? Math.round(priced.reduce((sum, item) => sum + (item.result.appliedMarginMicros ?? 0) * (item.result.finalSubtotalMinor ?? 0), 0) / weightedMarginBase) : null;
  return {
    services, totalMinor, totalHours, externalCostsMinor,
    effectiveHourlyMinor: totalMinor != null && totalHours > 0 ? Math.max(0, Math.round((totalMinor - externalCostsMinor) / totalHours)) : null,
    marginMicros,
    pricingTiers: { floorMinor: tierTotal("floor"), recommendedMinor: tierTotal("recommended"), premiumMinor: tierTotal("premium") },
    unpricedCount, isPartial: unpricedCount > 0 && priced.length > 0,
  };
}
