import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import type { PricingConfiguration } from "../../domain/types";
import { api } from "../../services/api";
import { EngineManager } from "./EngineManager";

const timestamp = "2026-08-10T00:00:00Z";
const pricing: PricingConfiguration = {
  definitions: [], parameters: [], options: [], rules: [], economicProfiles: [], marketSources: [], engineSources: [],
  engineCategories: [
    { id: "category-apparel", parentId: "category-products", slug: "apparel", name: "Indumentaria", engineType: "product", description: null, isSystem: true, sortOrder: 20, createdAt: timestamp, updatedAt: timestamp },
  ],
  pricingEngines: [
    { id: "engine-video", engineKey: "video-editing", name: "Edición de video", description: "Motor audiovisual", engineType: "service", categoryId: null, calculatorKey: "professional-service-v1", serviceDefinitionId: null, unitKind: "project", tagsJson: '["video"]', status: "active", classificationOrigin: "automatic", classificationConfidenceMicros: 1_000_000, classificationExplanation: "Motor del sistema.", classificationVersion: 1, isSystem: true, createdAt: timestamp, updatedAt: timestamp, archivedAt: null },
  ],
};

describe("EngineManager", () => {
  it("propone un motor de producto con reglas automáticas y permite corregirlo", async () => {
    vi.spyOn(api, "classifyPricingEngine").mockResolvedValueOnce({
      engineType: "product", categoryId: "category-apparel", categoryPath: ["Productos", "Indumentaria"],
      calculatorKey: "physical-product-v1", businessActivity: "Venta de remeras estampadas",
      pricingUnits: ["unidad", "lote"], suggestedCostTypes: ["prenda base", "estampado", "packaging"],
      suggestedSourceTypes: ["proveedores", "competidores"], tags: ["remeras", "indumentaria"],
      confidence: 0.93, explanation: "Se entrega un producto físico por unidad.",
      clarificationQuestion: null, aiAssisted: false, aiError: null,
    });
    render(<EngineManager pricing={pricing} onPricingChange={() => undefined} />);
    fireEvent.click(screen.getByRole("button", { name: /nuevo motor/i }));
    fireEvent.change(screen.getByLabelText(/Nombre de la actividad/i), { target: { value: "Venta de remeras estampadas" } });
    fireEvent.click(screen.getByRole("button", { name: /analizar actividad/i }));
    await waitFor(() => expect(screen.getByText("Se entrega un producto físico por unidad.")).toBeInTheDocument());
    expect(screen.getByText("93% de confianza · clasificador automático")).toBeInTheDocument();
    expect(screen.getByText("prenda base")).toBeInTheDocument();
    expect(screen.getByLabelText(/Tipo de motor/i)).toHaveValue("product");
    fireEvent.change(screen.getByLabelText(/Tipo de motor/i), { target: { value: "hybrid" } });
    expect(screen.getByText(/corregida por vos/i)).toBeInTheDocument();
  });
});
