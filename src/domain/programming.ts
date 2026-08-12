import type { ServiceConfigurationEnvelope } from "./types";
import {
  defaultProfessionalConfiguration,
  emptyExternalCost,
  parseProfessionalEnvelope,
  type ProfessionalServiceConfiguration,
} from "./professional";

export type ProgrammingConfiguration = ProfessionalServiceConfiguration;

export const defaultProgrammingConfiguration = defaultProfessionalConfiguration;

export function parseProgrammingEnvelope(json: string): ServiceConfigurationEnvelope<ProgrammingConfiguration> {
  return parseProfessionalEnvelope(json, "programming");
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

export { emptyExternalCost };
