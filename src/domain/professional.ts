import type { Currency, ServiceConfigurationEnvelope, ServiceType } from "./types";
import type { ExternalCost } from "./video";

export interface ProfessionalServiceConfiguration {
  parameterValues: Record<string, unknown>;
  externalCosts: ExternalCost[];
  notes: string;
}

export const defaultProfessionalConfiguration = (): ProfessionalServiceConfiguration => ({
  parameterValues: {}, externalCosts: [], notes: "",
});

export function parseProfessionalEnvelope(json: string, serviceType: ServiceType): ServiceConfigurationEnvelope<ProfessionalServiceConfiguration> {
  const parsed = JSON.parse(json) as ServiceConfigurationEnvelope<Partial<ProfessionalServiceConfiguration> & { category?: string }>;
  const data = parsed.data ?? {};
  return {
    schemaVersion: 2,
    serviceType,
    data: {
      ...defaultProfessionalConfiguration(),
      ...data,
      parameterValues: data.parameterValues ?? (data.category ? { projectType: data.category } : {}),
      externalCosts: data.externalCosts ?? [],
    },
  };
}

export function emptyExternalCost(currency: Currency): ExternalCost {
  return { id: crypto.randomUUID(), name: "", amountMinor: 0, currency, note: "" };
}
