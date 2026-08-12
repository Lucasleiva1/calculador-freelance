import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import type { EconomicProfile, PricingConfiguration, PricingEngine } from "../../domain/types";

const apiMock = vi.hoisted(() => ({ saveEconomyTemplate: vi.fn(), extractEconomyPdfText: vi.fn() }));
vi.mock("../../services/api", () => ({ api: apiMock }));

import { EconomySettings } from "./EconomySettings";

const stamp = "2026-08-11T00:00:00Z";
const engine = (id: string, engineKey: string, name: string): PricingEngine => ({ id, engineKey, name, description: null, engineType: "service", categoryId: null, calculatorKey: "professional-service-v1", serviceDefinitionId: null, unitKind: "hour", tagsJson: "[]", status: "active", classificationOrigin: "automatic", classificationConfidenceMicros: null, classificationExplanation: null, classificationVersion: 1, isSystem: true, createdAt: stamp, updatedAt: stamp, archivedAt: null });
const video = engine("engine-video-editing", "video-editing", "Edición de video");
const programming = engine("engine-programming", "programming", "Programación");
const videoProfile: EconomicProfile = { engineId: video.id, currency: "ARS", monthlyIncomeTargetMinor: null, monthlyExpensesMinor: null, billableHoursMicros: null, reserveTaxMicros: null, desiredMarginMicros: null, defaultUrgencyMicros: null, workDays: null, vacationWeeks: null, manualHourlyRateMinor: 35_000_00, updatedAt: stamp };
const pricing: PricingConfiguration = { definitions: [], parameters: [], options: [], rules: [], economicProfiles: [videoProfile], marketSources: [], engineCategories: [], pricingEngines: [video, programming], engineSources: [] };

describe("EconomySettings", () => {
  beforeEach(() => { apiMock.saveEconomyTemplate.mockReset(); });

  it("identifica arriba la profesión activa y mantiene perfiles separados", () => {
    render(<EconomySettings pricing={pricing} onSave={async () => undefined} initialCurrency="ARS" initialEngineKey="video-editing" />);
    expect(screen.getByRole("heading", { name: /estás configurando: edición de video/i })).toBeInTheDocument();
    expect(screen.getByText("Configurada", { selector: ".economy-profession__status > span" })).toBeInTheDocument();

    fireEvent.change(screen.getByRole("combobox", { name: /profesión o actividad/i }), { target: { value: programming.id } });

    expect(screen.getByRole("heading", { name: /estás configurando: programación/i })).toBeInTheDocument();
    expect(screen.getByText("Pendiente", { selector: ".economy-profession__status > span" })).toBeInTheDocument();
    expect(screen.getByLabelText(/tarifa manual de programación/i)).toHaveValue(null);
  });

  it("abre el guardado nativo para la guía y muestra la ubicación elegida", async () => {
    apiMock.saveEconomyTemplate.mockResolvedValue("C:\\Users\\demo\\Desktop\\guia-economia.md");
    render(<EconomySettings pricing={pricing} onSave={async () => undefined} initialCurrency="ARS" />);

    fireEvent.click(screen.getByRole("button", { name: /guardar guía para ia/i }));

    await waitFor(() => expect(apiMock.saveEconomyTemplate).toHaveBeenCalledWith("ai-guide", "Edición de video", "ARS"));
    expect(await screen.findByText(/Desktop\\guia-economia\.md/i)).toBeInTheDocument();
  });

  it("informa cuando se cancela el diálogo sin descargar silenciosamente", async () => {
    apiMock.saveEconomyTemplate.mockResolvedValue(null);
    render(<EconomySettings pricing={pricing} onSave={async () => undefined} />);

    fireEvent.click(screen.getByRole("button", { name: /guardar plantilla json/i }));

    expect(await screen.findByText(/cerraste el diálogo sin elegir/i)).toBeInTheDocument();
  });

  it("rechaza un archivo de otra profesión antes de aplicarlo", async () => {
    const { container } = render(<EconomySettings pricing={pricing} onSave={async () => undefined} initialEngineKey="programming" />);
    const input = container.querySelector('input[type="file"]') as HTMLInputElement;
    const file = new File(['{"actividad":"Edición de video","moneda":"ARS","tarifaManualPorHora":18000}'], "economia-video.json", { type: "application/json" });

    fireEvent.change(input, { target: { files: [file] } });

    expect(await screen.findByText(/corresponde a edición de video, pero estás configurando programación/i)).toBeInTheDocument();
    expect(screen.queryByRole("button", { name: /aplicar a programación/i })).not.toBeInTheDocument();
  });

  it("rechaza un archivo de otra moneda antes de aplicarlo", async () => {
    const { container } = render(<EconomySettings pricing={pricing} onSave={async () => undefined} initialEngineKey="programming" initialCurrency="ARS" />);
    const input = container.querySelector('input[type="file"]') as HTMLInputElement;
    const file = new File(['{"actividad":"Programación","moneda":"USD","tarifaManualPorHora":30}'], "economia-usd.json", { type: "application/json" });

    fireEvent.change(input, { target: { files: [file] } });

    expect(await screen.findByText(/está expresado en USD, pero el perfil abierto usa ARS/i)).toBeInTheDocument();
    expect(screen.queryByRole("button", { name: /aplicar a programación/i })).not.toBeInTheDocument();
  });
});
