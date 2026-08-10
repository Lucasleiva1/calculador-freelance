import { describe, expect, it } from "vitest";
import { calculateHybrid, calculateProduct, defaultHybridConfiguration, defaultProductConfiguration } from "./product";

const context = { currency: "USD" as const, hourlyRateMinor: 2_000, usdToArsMicros: 13_000_000 };

describe("motor de productos", () => {
  it("calcula piso, recomendado y premium con porcentajes dependientes del precio", () => {
    const result = calculateProduct({
      ...defaultProductConfiguration(), quantity: 20, commissionPercent: 10, taxPercent: 5,
      recommendedMarginPercent: 30, premiumMarginPercent: 45,
      costs: [{ id: "shirt", name: "Remera base", amountMinor: 1_000, currency: "USD", scope: "per_unit", category: "material" }],
    }, context);
    expect(result.status).toBe("ready");
    expect(result.productMetrics?.productionCostMinor).toBe(20_000);
    expect(result.pricingTiers?.floor.totalMinor).toBe(Math.ceil(20_000 / 0.85));
    expect(result.pricingTiers?.recommended.totalMinor).toBe(Math.ceil(20_000 / 0.55));
    expect(result.pricingTiers?.premium.totalMinor).toBe(Math.ceil(20_000 / 0.4));
  });

  it("conserva costos originales y queda pendiente si falta conversión", () => {
    const result = calculateProduct({ ...defaultProductConfiguration(), costs: [{ id: "ars", name: "Proveedor", amountMinor: 10_000, currency: "ARS", scope: "batch", category: "material" }] }, { ...context, usdToArsMicros: null });
    expect(result.status).toBe("incomplete");
    expect(result.issues.join(" ")).toContain("cambio USD/ARS");
  });

  it("combina servicio y producto sin duplicar el costo profesional", () => {
    const result = calculateHybrid({ ...defaultHybridConfiguration(), serviceHours: 5, costs: [{ id: "batch", name: "Producción", amountMinor: 10_000, currency: "USD", scope: "batch", category: "production" }] }, context);
    expect(result.status).toBe("ready");
    expect(result.productMetrics?.productionCostMinor).toBe(20_000);
    expect(result.lines.filter((line) => line.label === "Trabajo profesional")).toHaveLength(1);
  });

  it("aplica la merma al producto físico y no vuelve a cargar horas profesionales", () => {
    const result = calculateHybrid({
      ...defaultHybridConfiguration(), serviceHours: 5, wastePercent: 10,
      costs: [{ id: "batch", name: "Producción", amountMinor: 10_000, currency: "USD", scope: "batch", category: "production" }],
    }, context);
    expect(result.productMetrics?.productionCostMinor).toBe(21_000);
    expect(result.externalCostsMinor).toBe(11_000);
    expect(result.lines.find((line) => line.label === "Merma prevista")?.amountMinor).toBe(1_000);
  });
});
