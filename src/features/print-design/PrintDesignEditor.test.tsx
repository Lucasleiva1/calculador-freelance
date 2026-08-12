import { fireEvent, render, screen } from "@testing-library/react";
import { useState } from "react";
import { describe, expect, it } from "vitest";
import type { ParameterOption, PricingConfiguration, QuoteService, ServiceParameter } from "../../domain/types";
import type { ProfessionalServiceConfiguration } from "../../domain/professional";
import { PrintDesignEditor } from "./PrintDesignEditor";
import { printDesignSummary } from "../../domain/printDesign";

const stamp = "2026-08-12T00:00:00Z";
const parameter = (id: string, key: string, label: string, type: ServiceParameter["parameterType"], order: number): ServiceParameter => ({
  id, serviceDefinitionId: "service-print-design", parameterKey: key, name: label, label,
  parameterType: type, description: null, required: false, sortOrder: order, enabled: true,
  defaultValueJson: null, suggestionEnabled: false, isSystem: true, uiManaged: key === "estimatedHours",
  version: 1, createdAt: stamp, updatedAt: stamp,
});
const parameters = [
  parameter("main", "mainWorkType", "Tipo principal de trabajo", "single_select", 10),
  parameter("complex", "complexity", "Nivel de complejidad general", "single_select", 20),
  parameter("output", "printOutput", "Tipo de salida", "multi_select", 30),
  parameter("editable", "editableDelivery", "Entrega de archivo editable", "single_select", 40),
  parameter("hours", "estimatedHours", "Tiempo estimado", "number", 50),
];
const option = (id: string, parameterId: string, label: string, value: string): ParameterOption => ({ id, parameterId, label, value, sortOrder: 10, enabled: true, createdAt: stamp, updatedAt: stamp });
const options = [option("main-vector", "main", "Vectorización corregida manualmente", "vector-corrected"), option("complex-high", "complex", "Alto", "high"), option("output-dtf", "output", "DTF", "dtf"), option("editable-ai", "editable", "Sí, AI", "ai")];
const pricing: PricingConfiguration = {
  definitions: [{ id: "service-print-design", serviceType: "print-design", name: "Diseño de estampas", description: null, version: 1, enabled: true, suggestionsEnabled: true, defaultStrategy: "balanced", competitiveMarginMicros: null, balancedMarginMicros: null, premiumMarginMicros: null, createdAt: stamp, updatedAt: stamp }],
  parameters, options, rules: [], economicProfiles: [], marketSources: [], engineCategories: [], pricingEngines: [], engineSources: [],
};
const service: QuoteService = { id: "print", quoteId: "quote", serviceType: "print-design", title: "Estampa banda", sortOrder: 0, configurationVersion: 2, configurationJson: "{}", calculatedSubtotalMinor: null, suggestedSubtotalMinor: null, finalSubtotalMinor: null, hasOverride: false, manualSubtotalMinor: null, manualReason: null, pricingSnapshotJson: null, serviceDefinitionVersion: 1, rowRevision: 0, deletedAt: null, createdAt: stamp, updatedAt: stamp };

function Harness() {
  const [config, setConfig] = useState<ProfessionalServiceConfiguration>({ parameterValues: { mainWorkType: "vector-corrected", complexity: "high", printOutput: ["dtf"], editableDelivery: "ai" }, externalCosts: [], notes: "" });
  return <PrintDesignEditor service={service} clientName="Cliente prueba" config={config} pricing={pricing} suggestionsEnabled onChange={setConfig} />;
}

describe("PrintDesignEditor", () => {
  it("muestra la profesión y construye un resumen sin rubros ajenos", () => {
    render(<Harness />);
    expect(screen.getByText("Cliente prueba")).toBeInTheDocument();
    expect(screen.getByText(/Trabajo: Vectorización corregida manualmente/i)).toBeInTheDocument();
    expect(screen.getByText(/Salida: DTF/i)).toBeInTheDocument();
    expect(screen.queryByText(/serigraf/i)).not.toBeInTheDocument();
  });

  it("permite trabajar en días y convierte a horas para el cálculo", () => {
    render(<Harness />);
    fireEvent.change(screen.getByLabelText("Unidad de tiempo"), { target: { value: "days" } });
    fireEvent.change(screen.getByLabelText("Tiempo estimado"), { target: { value: "3" } });
    expect(screen.getByText(/3 días.*8 h por día.*24 h para el cálculo/i)).toBeInTheDocument();
    expect(screen.getByText(/Tiempo: 24 h 0 min/i)).toBeInTheDocument();
  });

  it("genera una descripción pública sólo con datos del alcance", () => {
    const summary = printDesignSummary({ mainWorkType: "vector-corrected", complexity: "high", printOutput: ["dtf"], estimatedHours: 2.25 }, parameters, options);
    expect(summary).toContain("Complejidad: Alto");
    expect(summary).toContain("Tiempo: 2 h 15 min");
  });
});
