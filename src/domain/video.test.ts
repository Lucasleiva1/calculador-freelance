import { describe, expect, it } from "vitest";
import { applyVideoPreset, calculateVideo, defaultVideoConfiguration, parseDuration } from "./video";

describe("Pricing V0 de Video", () => {
  it("suma base, costos externos y urgencia sin ocultar líneas", () => {
    const config = {
      ...defaultVideoConfiguration(),
      estimatedHours: 10,
      urgency: "priority" as const,
      urgencyFeeMinor: 1_000,
      externalCosts: [{ id: "1", name: "Música", amountMinor: 2_000, currency: "USD" as const, note: "" }],
    };
    const result = calculateVideo(config, { currency: "USD", hourlyRateMinor: 5_000, usdToArsMicros: null });
    expect(result.calculatedSubtotalMinor).toBe(53_000);
    expect(result.lines).toEqual([
      { label: "Horas × tarifa", amountMinor: 50_000 },
      { label: "Costos externos", amountMinor: 2_000 },
      { label: "Urgencia", amountMinor: 1_000 },
    ]);
  });

  it("convierte costos externos con la tasa manual", () => {
    const config = {
      ...defaultVideoConfiguration(), estimatedHours: 2,
      externalCosts: [{ id: "1", name: "Stock", amountMinor: 1_000, currency: "USD" as const, note: "" }],
    };
    const result = calculateVideo(config, { currency: "ARS", hourlyRateMinor: 100_000, usdToArsMicros: 12_000_000 });
    expect(result.externalCostsMinor).toBe(1_200_000);
    expect(result.calculatedSubtotalMinor).toBe(1_400_000);
  });

  it("no inventa un subtotal si falta la tasa de cambio", () => {
    const config = {
      ...defaultVideoConfiguration(), estimatedHours: 1,
      externalCosts: [{ id: "1", name: "Stock", amountMinor: 1_000, currency: "USD" as const, note: "" }],
    };
    const result = calculateVideo(config, { currency: "ARS", hourlyRateMinor: 100_000, usdToArsMicros: null });
    expect(result.calculatedSubtotalMinor).toBeNull();
    expect(result.issues.join(" ")).toContain("cambio USD/ARS");
  });

  it("conserva el calculado cuando existe un override", () => {
    const config = { ...defaultVideoConfiguration(), estimatedHours: 4 };
    const result = calculateVideo(config, { currency: "USD", hourlyRateMinor: 5_000, usdToArsMicros: null }, 30_000);
    expect(result.calculatedSubtotalMinor).toBe(20_000);
    expect(result.effectiveSubtotalMinor).toBe(30_000);
  });
});

describe("Configuración de Video", () => {
  it("valida duraciones MM:SS", () => {
    expect(parseDuration("08:30")).toBe(510);
    expect(parseDuration("25:00")).toBe(1500);
    expect(parseDuration("08:75")).toBeNull();
  });

  it("un preset no pisa valores económicos", () => {
    const config = { ...defaultVideoConfiguration(), estimatedHours: 8, urgencyFeeMinor: 900, externalCosts: [{ id: "1", name: "Licencia", amountMinor: 500, currency: "USD" as const, note: "" }] };
    const applied = applyVideoPreset(config, JSON.stringify({ pieceType: "youtube", estimatedHours: 99, urgencyFeeMinor: 99, externalCosts: [] }));
    expect(applied.pieceType).toBe("youtube");
    expect(applied.estimatedHours).toBe(8);
    expect(applied.urgencyFeeMinor).toBe(900);
    expect(applied.externalCosts).toHaveLength(1);
  });
});

