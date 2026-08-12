import type { Currency, MarketObservation, MarketSnapshot, QuoteService, SuggestionStrategy } from "./types";

export interface AutomaticPriceSummary {
  minimumFilteredMinor: number | null;
  p25Minor: number | null;
  medianMinor: number | null;
  p75Minor: number | null;
  maximumFilteredMinor: number | null;
  confidenceLevel: "HIGH" | "MEDIUM" | "LOW" | "INSUFFICIENT";
  comparableCount: number;
  sourceCount: number;
  recentCount: number;
  salaryExcludedCount: number;
  explanations: string[];
}

export interface AutomaticPriceOption {
  summary: AutomaticPriceSummary;
  suggestedPriceMinor: number | null;
  region: "AR" | "INTERNATIONAL";
}

export interface ThreePriceSnapshot {
  market: AutomaticPriceOption | null;
  international: AutomaticPriceOption | null;
  fxRateMicros: number | null;
  fxRateDate: string | null;
  fxRateSource: string | null;
}

export function parseThreePriceSnapshot(snapshot: MarketSnapshot | null): ThreePriceSnapshot {
  const empty = { market: null, international: null, fxRateMicros: null, fxRateDate: null, fxRateSource: null };
  if (!snapshot) return empty;
  try {
    const value = JSON.parse(snapshot.summaryJson) as {
      pricingOptions?: { market?: AutomaticPriceOption; international?: AutomaticPriceOption };
      fxRateMicros?: number | null;
      fxRateDate?: string | null;
      fxRateSource?: string | null;
    };
    return {
      market: value.pricingOptions?.market ?? null,
      international: value.pricingOptions?.international ?? null,
      fxRateMicros: value.fxRateMicros ?? null,
      fxRateDate: value.fxRateDate ?? null,
      fxRateSource: value.fxRateSource ?? null,
    };
  } catch {
    return empty;
  }
}

export interface MarketQueryContext {
  service: string;
  subtype: string | null;
  regionTargets: string[];
  level: string | null;
  durationMinutes: number | null;
  quantity: number | null;
  estimatedHours: number | null;
  features: string[];
}

function durationMinutes(value: unknown) {
  if (typeof value !== "string") return null;
  const match = /^(\d{1,3}):([0-5]\d)$/.exec(value);
  return match ? Number(match[1]) + Number(match[2]) / 60 : null;
}

export function buildMarketQueryContext(service: QuoteService, scope: "argentina" | "international" | "both" | null): MarketQueryContext {
  const envelope = JSON.parse(service.configurationJson) as { data?: Record<string, unknown> & { parameterValues?: Record<string, unknown> } };
  const data = envelope.data ?? {};
  const values = data.parameterValues ?? data;
  const string = (key: string) => typeof values[key] === "string" && values[key] ? String(values[key]) : null;
  const number = (key: string) => typeof values[key] === "number" && Number.isFinite(values[key]) ? Number(values[key]) : null;
  const regions = scope === "argentina" ? ["AR"] : scope === "international" ? ["INTERNATIONAL"] : ["AR", "LATAM", "INTERNATIONAL"];
  const allowedFeatures = service.serviceType === "video-editing"
    ? new Set(["resolution", "editingLevel", "revisions", "urgency", "formats", "color", "audio", "subtitles", "videoAi", "voiceAi", "soundAi", "backgroundRemoval", "motion", "broll", "additionalVersions"])
    : service.serviceType === "print-design"
      ? new Set(["mainWorkType", "additionalOperations", "complexity", "inputQuality", "backgroundLevel", "restorationLevel", "vectorizationLevel", "compositionLevel", "aiLevel", "typographyLevel", "colorLevel", "printOutput", "halftoneLevel", "elementCountBand", "initialProposals", "includedRevisions", "variantLevel", "editableDelivery", "urgency", "designOrigin"])
      : new Set(["projectType", "frontend", "backend", "database", "auth", "integrations", "screens", "responsive", "deploy", "ai", "complexity"]);
  const features = Object.entries(values).filter(([key, value]) => allowedFeatures.has(key) && (value === true || (Array.isArray(value) && value.length > 0) || (typeof value === "string" && !["", "none", "basic", "normal"].includes(value)))).map(([key]) => key);
  return {
    service: service.serviceType,
    subtype: string(service.serviceType === "video-editing" ? "pieceType" : service.serviceType === "print-design" ? "mainWorkType" : "projectType"),
    regionTargets: regions,
    level: string(service.serviceType === "video-editing" ? "editingLevel" : "complexity"),
    durationMinutes: durationMinutes(values.finalDuration),
    quantity: number("quantity"),
    estimatedHours: number("estimatedHours"),
    features,
  };
}

export function observationAmount(observation: MarketObservation) {
  if (observation.priceValueMinor != null) return observation.priceValueMinor;
  if (observation.priceMinMinor != null && observation.priceMaxMinor != null) return Math.round((observation.priceMinMinor + observation.priceMaxMinor) / 2);
  return observation.priceMinMinor ?? observation.priceMaxMinor;
}

export function suggestedFromSnapshot(snapshot: MarketSnapshot, strategy: SuggestionStrategy) {
  if (snapshot.confidenceLevel === "INSUFFICIENT") return null;
  const target = strategy === "competitive" ? snapshot.p25Minor : strategy === "premium" ? snapshot.p75Minor : snapshot.marketMedianMinor;
  if (target == null) return null;
  if (snapshot.calculatedPriceMinor == null) return target;
  return Math.max(snapshot.calculatedPriceMinor, Math.round(snapshot.calculatedPriceMinor * 0.4 + target * 0.6));
}

export function marketRangeLabel(snapshot: MarketSnapshot | null, currency: Currency) {
  if (!snapshot || snapshot.minimumFilteredMinor == null || snapshot.maximumFilteredMinor == null) return null;
  return { currency, low: snapshot.minimumFilteredMinor, high: snapshot.maximumFilteredMinor, median: snapshot.marketMedianMinor };
}

export interface MarketTextInterpreter { interpret(text: string): Array<{ label: string; value: string }>; }

export class DeterministicMarketInterpreter implements MarketTextInterpreter {
  interpret(text: string) {
    const prices = text.match(/(?:USD|ARS|US\$|\$|£|€)\s*[\d.,]+/g) ?? [];
    return prices.slice(0, 10).map((value) => ({ label: "Precio detectado", value }));
  }
}

// Contrato preparado; la implementación dependerá del runtime local que el usuario elija.
export interface LocalAIInterpreter extends MarketTextInterpreter {
  readonly runtimeId: string;
}
