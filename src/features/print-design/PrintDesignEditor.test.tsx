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
  parameter("product", "productType", "Producto", "single_select", 10),
  parameter("system", "printSystem", "Sistema", "single_select", 20),
  parameter("tasks", "workTasks", "Trabajo", "multi_select", 30),
  parameter("complex", "complexity", "Nivel de complejidad general", "single_select", 20),
  parameter("delivery", "deliveryExtras", "Entrega", "multi_select", 40),
  parameter("hours", "estimatedHours", "Tiempo estimado", "number", 50),
];
const option = (id: string, parameterId: string, label: string, value: string): ParameterOption => ({ id, parameterId, label, value, sortOrder: 10, enabled: true, createdAt: stamp, updatedAt: stamp });
const options = [option("product-shirt", "product", "Remera", "shirt"), option("system-dtf", "system", "DTF", "dtf"), option("task-vector", "tasks", "Vectorizar texto o gráfico simple", "vectorize-simple"), option("complex-high", "complex", "Compleja", "complex"), option("delivery-ai", "delivery", "AI / vector editable", "ai-vector")];
const pricing: PricingConfiguration = {
  definitions: [{ id: "service-print-design", serviceType: "print-design", name: "Diseño de estampas", description: null, version: 2, enabled: true, suggestionsEnabled: true, defaultStrategy: "balanced", competitiveMarginMicros: null, balancedMarginMicros: null, premiumMarginMicros: null, createdAt: stamp, updatedAt: stamp }],
  parameters, options, rules: [], economicProfiles: [], marketSources: [], engineCategories: [], pricingEngines: [], engineSources: [],
};
const service: QuoteService = { id: "print", quoteId: "quote", serviceType: "print-design", title: "Estampa banda", sortOrder: 0, configurationVersion: 3, configurationJson: "{}", calculatedSubtotalMinor: null, suggestedSubtotalMinor: null, finalSubtotalMinor: null, hasOverride: false, manualSubtotalMinor: null, manualReason: null, pricingSnapshotJson: null, serviceDefinitionVersion: 2, rowRevision: 0, deletedAt: null, createdAt: stamp, updatedAt: stamp };

function Harness() {
  const [config, setConfig] = useState<ProfessionalServiceConfiguration>({ parameterValues: { hasReference: true, materialType: "ready", clientTier: "medium", productType: "shirt", garmentTone: "dark", printSystem: "dtf", workTasks: ["vectorize-simple"], complexity: "basic", complexityMode: "automatic", deliveryExtras: ["ai-vector"], estimatedHours: 2.25, estimatedHoursMode: "manual" }, externalCosts: [], notes: "" });
  return <PrintDesignEditor service={service} clientName="Cliente prueba" config={config} pricing={pricing} suggestionsEnabled onChange={setConfig} />;
}

describe("PrintDesignEditor", () => {
  it("muestra la profesión y construye un resumen sin rubros ajenos", () => {
    render(<Harness />);
    expect(screen.getByText("Cliente prueba")).toBeInTheDocument();
    expect(screen.getByText(/Trabajo: Vectorizar texto o gráfico simple/i)).toBeInTheDocument();
    expect(screen.getByText(/Sistema: DTF/i)).toBeInTheDocument();
    expect(screen.queryByText(/serigraf/i)).not.toBeInTheDocument();
  });

  it("permite trabajar en días y convierte a horas para el cálculo", () => {
    render(<Harness />);
    fireEvent.change(screen.getByLabelText("Unidad de tiempo"), { target: { value: "days" } });
    fireEvent.change(screen.getByLabelText("Tiempo estimado"), { target: { value: "3" } });
    expect(screen.getByText(/3 días.*8 h por día.*24 h para el cálculo/i)).toBeInTheDocument();
    expect(screen.getByText(/Tiempo: 24 h/i)).toBeInTheDocument();
  });

  it("genera una descripción pública sólo con datos del alcance", () => {
    const summary = printDesignSummary({ hasReference: true, productType: "shirt", printSystem: "dtf", workTasks: ["vectorize-simple"], complexity: "complex", complexityMode: "manual", estimatedHours: 2.25, estimatedHoursMode: "manual", deliveryExtras: ["ai-vector"] }, parameters, options);
    expect(summary).toContain("Complejidad: Compleja");
    expect(summary).toContain("Tiempo: 2,25 h");
    expect(summary).toContain("Entrega: archivo final + AI / vector editable");
  });

  it("al elegir sin referencia oculta el material y activa crear desde cero", () => {
    render(<Harness />);
    expect(screen.getByText("Material recibido *")).toBeInTheDocument();

    fireEvent.click(screen.getByRole("radio", { name: /No, hay que crear desde cero/i }));

    expect(screen.queryByText("Material recibido *")).not.toBeInTheDocument();
    expect(screen.getByRole("checkbox", { name: /Crear diseño desde cero/i })).toBeChecked();
    expect(screen.getByText("Compleja")).toBeInTheDocument();
  });
});
