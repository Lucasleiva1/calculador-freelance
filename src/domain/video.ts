import { convertMinor } from "./money";
import type {
  Currency,
  PricingContext,
  ServiceConfigurationEnvelope,
  ServiceModuleDefinition,
  ServiceResult,
} from "./types";

export interface ExternalCost {
  id: string;
  name: string;
  amountMinor: number;
  currency: Currency;
  note: string;
}

export interface VideoConfiguration {
  pieceType: string;
  quantity: number;
  rawMinutes: number | null;
  finalDuration: string;
  resolution: "1080p";
  editingLevel: "basic" | "professional" | "advanced" | "custom";
  revisions: number;
  urgency: "normal" | "priority" | "48h" | "24h";
  urgencyFeeMinor: number;
  formats: Array<"16:9" | "9:16" | "1:1">;
  estimatedHours: number | null;
  color: "none" | "basic" | "look";
  audio: "basic" | "cleanup" | "music-effects" | "sound-design";
  subtitles: "none" | "standard" | "designed";
  videoAi: "none" | "partial" | "important";
  voiceAi: boolean;
  soundAi: boolean;
  backgroundRemoval: boolean;
  motion: "none" | "basic" | "ai-assisted" | "custom";
  broll: "client" | "simple" | "advanced";
  additionalVersions: number;
  externalCosts: ExternalCost[];
}

export const defaultVideoConfiguration = (): VideoConfiguration => ({
  pieceType: "",
  quantity: 1,
  rawMinutes: null,
  finalDuration: "",
  resolution: "1080p",
  editingLevel: "basic",
  revisions: 1,
  urgency: "normal",
  urgencyFeeMinor: 0,
  formats: [],
  estimatedHours: null,
  color: "none",
  audio: "basic",
  subtitles: "none",
  videoAi: "none",
  voiceAi: false,
  soundAi: false,
  backgroundRemoval: false,
  motion: "none",
  broll: "client",
  additionalVersions: 0,
  externalCosts: [],
});

export function parseDuration(value: string): number | null {
  if (!value) return null;
  const match = /^(\d{1,3}):([0-5]\d)$/.exec(value.trim());
  if (!match) return null;
  return Number(match[1]) * 60 + Number(match[2]);
}

export function validateVideo(config: VideoConfiguration): string[] {
  const issues: string[] = [];
  if (!Number.isInteger(config.quantity) || config.quantity < 1) issues.push("La cantidad debe ser 1 o mayor.");
  if (config.rawMinutes != null && (!Number.isFinite(config.rawMinutes) || config.rawMinutes < 0)) issues.push("El material bruto no es válido.");
  if (config.finalDuration && parseDuration(config.finalDuration) == null) issues.push("Usá el formato MM:SS para la duración final.");
  if (config.estimatedHours != null && (!Number.isFinite(config.estimatedHours) || config.estimatedHours < 0)) issues.push("Las horas estimadas no son válidas.");
  if (!Number.isInteger(config.revisions) || config.revisions < 0) issues.push("Las revisiones no son válidas.");
  if (!Number.isInteger(config.additionalVersions) || config.additionalVersions < 0) issues.push("Las versiones adicionales no son válidas.");
  if (!Number.isFinite(config.urgencyFeeMinor) || config.urgencyFeeMinor < 0) issues.push("El importe de urgencia no es válido.");
  for (const cost of config.externalCosts) {
    if (!cost.name.trim()) issues.push("Cada costo externo necesita un nombre.");
    if (!Number.isFinite(cost.amountMinor) || cost.amountMinor < 0) issues.push("Hay un costo externo inválido.");
  }
  return issues;
}

export function calculateVideo(
  config: VideoConfiguration,
  context: PricingContext,
  manualSubtotalMinor: number | null = null,
): ServiceResult {
  const validationIssues = validateVideo(config);
  const issues = [...validationIssues];
  const lines = [];
  const hours = config.estimatedHours;
  if (hours == null) issues.push("Indicá las horas estimadas.");
  if (context.hourlyRateMinor == null) issues.push(`Configurá tu tarifa base en ${context.currency}.`);

  const base = hours != null && context.hourlyRateMinor != null
    ? Math.round(hours * context.hourlyRateMinor)
    : null;
  if (base != null) lines.push({ label: "Horas × tarifa", amountMinor: base });

  let externalCostsMinor = 0;
  let conversionMissing = false;
  for (const cost of config.externalCosts) {
    const converted = convertMinor(cost.amountMinor, cost.currency, context.currency, context.usdToArsMicros);
    if (converted == null) {
      conversionMissing = true;
    } else {
      externalCostsMinor += converted;
    }
  }
  if (conversionMissing) issues.push("Configurá el cambio USD/ARS para convertir los costos externos.");
  if (externalCostsMinor > 0) lines.push({ label: "Costos externos", amountMinor: externalCostsMinor });
  if (config.urgencyFeeMinor > 0) lines.push({ label: "Urgencia", amountMinor: config.urgencyFeeMinor });

  const canCalculate = base != null && !conversionMissing && validationIssues.length === 0;
  const calculatedSubtotalMinor = canCalculate
    ? base + externalCostsMinor + config.urgencyFeeMinor
    : null;
  return {
    status: calculatedSubtotalMinor == null ? "incomplete" : "ready",
    calculatedSubtotalMinor,
    suggestedSubtotalMinor: null,
    finalSubtotalMinor: manualSubtotalMinor ?? calculatedSubtotalMinor,
    effectiveSubtotalMinor: manualSubtotalMinor ?? calculatedSubtotalMinor,
    hasOverride: manualSubtotalMinor != null,
    hours,
    externalCostsMinor,
    effectiveHourlyMinor: calculatedSubtotalMinor != null && hours != null && hours > 0
      ? Math.round(((manualSubtotalMinor ?? calculatedSubtotalMinor) - externalCostsMinor) / hours)
      : null,
    appliedMarginMicros: null,
    lines,
    issues: [...new Set(issues)],
  };
}

export function videoSummary(config: VideoConfiguration): string[] {
  return [
    config.pieceType || "Tipo pendiente",
    `${config.quantity} ${config.quantity === 1 ? "pieza" : "piezas"}`,
    "Full HD",
    config.formats.join(" · ") || "Formato pendiente",
  ];
}

export const videoModule: ServiceModuleDefinition<VideoConfiguration> = {
  type: "video-editing",
  label: "Edición de video",
  schemaVersion: 1,
  createDefaultConfiguration: defaultVideoConfiguration,
  validate: validateVideo,
  calculate: calculateVideo,
  summarize: videoSummary,
};

export function parseVideoEnvelope(json: string): ServiceConfigurationEnvelope<VideoConfiguration> {
  const parsed = JSON.parse(json) as ServiceConfigurationEnvelope<Partial<VideoConfiguration>>;
  return {
    schemaVersion: parsed.schemaVersion ?? 1,
    serviceType: "video-editing",
    data: { ...defaultVideoConfiguration(), ...parsed.data, externalCosts: parsed.data.externalCosts ?? [] },
  };
}

const presetEconomicKeys = new Set(["estimatedHours", "externalCosts", "urgencyFeeMinor"]);

export function applyVideoPreset(config: VideoConfiguration, presetJson: string): VideoConfiguration {
  const parsed = JSON.parse(presetJson) as Partial<VideoConfiguration>;
  const safePreset = Object.fromEntries(Object.entries(parsed).filter(([key]) => !presetEconomicKeys.has(key)));
  return { ...config, ...safePreset };
}
