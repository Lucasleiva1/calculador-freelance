import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import type { PricingConfiguration } from "../../domain/types";

const apiMock = vi.hoisted(() => ({ saveEconomyTemplate: vi.fn(), extractEconomyPdfText: vi.fn() }));
vi.mock("../../services/api", () => ({ api: apiMock }));

import { EconomySettings } from "./EconomySettings";

const pricing: PricingConfiguration = { definitions: [], parameters: [], options: [], rules: [], economicProfiles: [], marketSources: [], engineCategories: [], pricingEngines: [], engineSources: [] };

describe("EconomySettings", () => {
  beforeEach(() => { apiMock.saveEconomyTemplate.mockReset(); });

  it("abre el guardado nativo para la guía y muestra la ubicación elegida", async () => {
    apiMock.saveEconomyTemplate.mockResolvedValue("C:\\Users\\demo\\Desktop\\guia-economia.md");
    render(<EconomySettings pricing={pricing} onSave={async () => undefined} initialCurrency="ARS" />);

    fireEvent.click(screen.getByRole("button", { name: /guardar guía para ia/i }));

    await waitFor(() => expect(apiMock.saveEconomyTemplate).toHaveBeenCalledWith("ai-guide"));
    expect(await screen.findByText(/Desktop\\guia-economia\.md/i)).toBeInTheDocument();
  });

  it("informa cuando se cancela el diálogo sin descargar silenciosamente", async () => {
    apiMock.saveEconomyTemplate.mockResolvedValue(null);
    render(<EconomySettings pricing={pricing} onSave={async () => undefined} />);

    fireEvent.click(screen.getByRole("button", { name: /guardar plantilla json/i }));

    expect(await screen.findByText(/cerraste el diálogo sin elegir/i)).toBeInTheDocument();
  });
});
