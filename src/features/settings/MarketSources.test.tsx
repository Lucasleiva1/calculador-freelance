import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import type { MarketSource, PricingConfiguration } from "../../domain/types";
import { api } from "../../services/api";
import { MarketSources } from "./MarketSources";

const source: MarketSource = {
  id: "source-test", name: "Tarifario de prueba", baseUrl: "https://example.com/rates",
  sourceType: "professional_tariff", regionsJson: '["AR"]', supportedServicesJson: '["video-editing"]',
  priority: 10, enabled: true, usageMode: "market_price", acquisitionMode: "manual", cooldownHours: 24,
  notes: null, isSystemSource: true, systemKey: "test", defaultDataJson: null,
  purpose: "Ofrece aranceles creativos argentinos.", dataContribution: "Aporta precio por minuto y tipo de cliente.",
  appBenefit: "Contrasta el cálculo de video sin cambiar el final.", participatesInSuggestions: true,
  automationStatus: "BLOCKED", currentStatus: "BLOCKED", adapterKey: "tarifario",
  lastRequestAt: null, lastSuccessAt: null, lastFailureAt: null, cooldownUntil: null,
  consecutiveFailures: 0, lastHttpStatus: null, lastError: "Sitio suspendido.", observationCount: 0,
  archivedAt: null, businessSourceType: "market", marketCountry: "Argentina", sourceCurrency: "ARS",
  sourceUpdatedAt: "2026-08-10", classificationOrigin: "automatic", classificationJson: null,
  createdAt: "2026-08-10T00:00:00Z", updatedAt: "2026-08-10T00:00:00Z",
};

const pricing: PricingConfiguration = { definitions: [], parameters: [], options: [], rules: [], economicProfiles: [], marketSources: [source], engineCategories: [], pricingEngines: [{ id: "engine-shirts", engineKey: "venta-remeras", name: "Venta de remeras", description: null, engineType: "product", categoryId: null, calculatorKey: "physical-product-v1", serviceDefinitionId: null, unitKind: "unit", tagsJson: '["remeras"]', status: "active", classificationOrigin: "automatic", classificationConfidenceMicros: 900_000, classificationExplanation: null, classificationVersion: 1, isSystem: false, createdAt: "2026-08-10T00:00:00Z", updatedAt: "2026-08-10T00:00:00Z", archivedAt: null }], engineSources: [] };

describe("MarketSources", () => {
  it("explica qué ofrece, qué dato aporta y cómo ayuda cada fuente", () => {
    render(<MarketSources pricing={pricing} onPricingChange={() => undefined} />);
    expect(screen.getByText(source.purpose!)).toBeInTheDocument();
    expect(screen.getByText(source.dataContribution!)).toBeInTheDocument();
    expect(screen.getByText(source.appBenefit!)).toBeInTheDocument();
    expect(screen.getByText("Sitio suspendido.")).toBeInTheDocument();
  });

  it("inicia una fuente nueva como manual y no revisada", () => {
    render(<MarketSources pricing={pricing} onPricingChange={() => undefined} />);
    fireEvent.click(screen.getByRole("button", { name: /agregar fuente/i }));
    expect(screen.getByText(/Una fuente nueva empieza en modo manual/i)).toBeInTheDocument();
    expect(screen.getByRole("button", { name: /clasificar fuente/i })).toBeDisabled();
  });

  it("clasifica una fuente y preselecciona motores sin delegar el precio", async () => {
    vi.spyOn(api, "classifyMarketSource").mockResolvedValueOnce({ businessSourceType: "supplier", suggestedEngineTypes: ["product", "hybrid"], role: "cost_input", tags: ["remeras"], confidence: 0.91, explanation: "Aporta costos directos de prendas.", aiAssisted: false, aiError: null });
    render(<MarketSources pricing={pricing} onPricingChange={() => undefined} />);
    fireEvent.click(screen.getByRole("button", { name: /agregar fuente/i }));
    fireEvent.change(screen.getByLabelText(/^Nombre/i), { target: { value: "Proveedor de remeras" } });
    fireEvent.click(screen.getByRole("button", { name: /clasificar fuente/i }));
    await waitFor(() => expect(screen.getByText(/Aporta costos directos de prendas/i)).toBeInTheDocument());
    expect(screen.getByLabelText(/Categoría de fuente/i)).toHaveValue("supplier");
    expect(screen.getByLabelText(/Rol en los motores/i)).toHaveValue("cost_input");
    expect(screen.getByRole("checkbox", { name: /Venta de remeras/i })).toBeChecked();
  });
});
