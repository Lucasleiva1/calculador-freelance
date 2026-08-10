import type { AppSettings, PricingConfiguration, QuoteService, ServiceResult, Workspace } from "./types";
import { parseProgrammingEnvelope } from "./programming";
import { parseVideoEnvelope } from "./video";
import { emptyResult, resultFromService, runPricingEngine } from "./pricingEngine";

export interface EvaluatedService { service: QuoteService; result: ServiceResult; }
export interface ProjectResult { services: EvaluatedService[]; totalMinor: number | null; totalHours: number; externalCostsMinor: number; effectiveHourlyMinor: number | null; unpricedCount: number; isPartial: boolean; }

function liveResult(service: QuoteService, workspace: Workspace, settings: AppSettings, pricing: PricingConfiguration) {
  try {
    if (service.serviceType === "video-editing") {
      const config = parseVideoEnvelope(service.configurationJson).data;
      return runPricingEngine({ serviceType: service.serviceType, currency: workspace.quote.currency, parameterValues: config as unknown as Record<string, unknown>, externalCosts: config.externalCosts, fixedUrgencyMinor: config.urgencyFeeMinor, finalOverrideMinor: service.finalSubtotalMinor ?? service.manualSubtotalMinor, hasOverride: service.hasOverride || service.manualSubtotalMinor != null, settings, pricing });
    }
    const config = parseProgrammingEnvelope(service.configurationJson).data;
    return runPricingEngine({ serviceType: service.serviceType, currency: workspace.quote.currency, parameterValues: config.parameterValues, externalCosts: config.externalCosts, finalOverrideMinor: service.finalSubtotalMinor ?? service.manualSubtotalMinor, hasOverride: service.hasOverride || service.manualSubtotalMinor != null, settings, pricing });
  } catch { return emptyResult("La configuración guardada no se pudo leer."); }
}

export function evaluateWorkspace(workspace: Workspace, settings: AppSettings, pricing: PricingConfiguration): ProjectResult {
  const services = workspace.services.map((service): EvaluatedService => ({ service, result: resultFromService(service) ?? liveResult(service, workspace, settings, pricing) }));
  const priced = services.filter(({ result }) => result.finalSubtotalMinor != null);
  const unpricedCount = services.length - priced.length;
  const totalMinor = priced.length === 0 ? null : priced.reduce((sum, item) => sum + (item.result.finalSubtotalMinor ?? 0), 0);
  const totalHours = services.reduce((sum, item) => sum + (item.result.hours ?? 0), 0);
  const externalCostsMinor = services.reduce((sum, item) => sum + item.result.externalCostsMinor, 0);
  return { services, totalMinor, totalHours, externalCostsMinor, effectiveHourlyMinor: totalMinor != null && totalHours > 0 ? Math.round((totalMinor - externalCostsMinor) / totalHours) : null, unpricedCount, isPartial: unpricedCount > 0 && priced.length > 0 };
}
