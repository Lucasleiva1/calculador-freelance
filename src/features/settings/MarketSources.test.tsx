import { fireEvent, render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import type { MarketSource, PricingConfiguration } from "../../domain/types";
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
  archivedAt: null, createdAt: "2026-08-10T00:00:00Z", updatedAt: "2026-08-10T00:00:00Z",
};

const pricing: PricingConfiguration = { definitions: [], parameters: [], options: [], rules: [], economicProfiles: [], marketSources: [source] };

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
    expect(screen.getByText(/MANUAL \+ UNREVIEWED/i)).toBeInTheDocument();
    expect(screen.getByLabelText("Método de adquisición")).toBeDisabled();
  });
});
