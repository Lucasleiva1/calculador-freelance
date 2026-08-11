import type { Currency, ServiceConfigurationEnvelope } from "./types";
import type { ExternalCost } from "./video";

export interface ProgrammingConfiguration {
  parameterValues: Record<string, unknown>;
  externalCosts: ExternalCost[];
  notes: string;
}

export const defaultProgrammingConfiguration = (): ProgrammingConfiguration => ({
  parameterValues: {}, externalCosts: [], notes: "",
});

export function parseProgrammingEnvelope(json: string): ServiceConfigurationEnvelope<ProgrammingConfiguration> {
  const parsed = JSON.parse(json) as ServiceConfigurationEnvelope<Partial<ProgrammingConfiguration> & { category?: string }>;
  const data = parsed.data ?? {};
  return {
    schemaVersion: 2,
    serviceType: "programming",
    data: {
      ...defaultProgrammingConfiguration(),
      ...data,
      parameterValues: data.parameterValues ?? (data.category ? { projectType: data.category } : {}),
      externalCosts: data.externalCosts ?? [],
    },
  };
}

export function programmingSummary(config: ProgrammingConfiguration) {
  const type = String(config.parameterValues.projectType || "Tipo pendiente");
  const hours = Number(config.parameterValues.estimatedHours || 0);
  const amount = Number(config.parameterValues.effortAmount || 0);
  const unit = config.parameterValues.effortUnit;
  const effort = amount > 0 && unit === "days" ? `${amount} ${amount === 1 ? "día" : "días"}`
    : amount > 0 && unit === "weeks" ? `${amount} ${amount === 1 ? "semana" : "semanas"}`
      : hours > 0 ? `${hours} h` : "Tiempo pendiente";
  return [type, effort];
}

export function emptyExternalCost(currency: Currency): ExternalCost {
  return { id: crypto.randomUUID(), name: "", amountMinor: 0, currency, note: "" };
}
